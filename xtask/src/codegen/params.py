import pandas as pd
import re
import sys
from glob import glob
from pathlib import Path
from itertools import zip_longest
from textwrap import dedent


SNAKECASE_RE = re.compile(r'(?!^)([A-Z]+)')
SNAKECASE_CLEAN_RE = re.compile(r'_+')
SLUG_RE = re.compile(r'([^a-zA-Z]+)')

PARAM_VTABLE_TEMPLATE = '''
type BoxedVisitorLambda = Box<dyn Fn(*const c_void, &mut dyn ParamVisitor) + Send + Sync>;

pub static PARAM_VTABLE: SyncLazy<HashMap<String, BoxedVisitorLambda>> = SyncLazy::new(|| {{
    [
        {vtable_fields}
    ].into_iter().collect()
}});'''

STRUCT_TEMPLATE = '''
    #[derive(ParamStruct, Debug)]
    #[repr(C)]
    pub struct {param_name} {{
        {fields}
    }}
'''

FIELD_TEMPLATE = '''
        pub {field_name}: {field_type},
'''.strip()

def to_snake_case(s):
    return SNAKECASE_CLEAN_RE.sub('_', SNAKECASE_RE.sub(r'_\1', s).lower())


def to_camel_case(s):
    return ''.join(i.title() for i in s.split('_'))


def to_slug(s):
    return SLUG_RE.sub('', s).lower()


def build_param_layouts(paramdex_path, xtask_path):
    paramdex_path = Path(paramdex_path)
    xtask_path = Path(xtask_path)

    # Param layouts -- Credits: Soulsmodding community's Paramdex
    xml_files = dict(
        (to_slug(Path(i).stem.replace('_ST', '')), Path(i).resolve())
        for i in (paramdex_path / 'ER/Defs').glob('*.xml')
    )

    # Param names from the game's memory
    param_names = dict(
        (to_slug(i), i)
        for i in map(lambda x: Path(x).stem.replace('_ST', ''), (paramdex_path / 'ER/Defs').glob('*.xml'))
    )

    # Currently broken
    del xml_files['defaultkeyassign']
    del param_names['defaultkeyassign']

    assert(xml_files.keys() == param_names.keys())

    return [
        ParamLayout(name=param_names[i], layout=xml_files[i])
        for i in param_names.keys()
    ]


class ParamLayout:
    def __init__(self, name, layout):
        self.name = name
        # self.name = to_camel_case(pd.read_xml(layout)['ParamType'][0])
        self.name_snake_case = to_snake_case(name)
        self.fields = ParamLayout.dedup_fields(ParamLayout.group_bitfields([
            Field(i) for i in pd.read_xml(layout, xpath='./Fields/*')['Def']
        ]))

    def get_struct(self):
        fields = '\n        '.join(
            field.format()
            for field in self.fields
        )
        return STRUCT_TEMPLATE.format(param_name=self.name, fields=fields)

    @staticmethod
    def fix_name(name: str):
        if name[0].isdigit():
            return 'field' + name

        if name == 'type':
            return 'ty'

        return name

    @staticmethod
    def dedup_fields(fields):
        fieldset = set()
        idx = 0
        for f in fields:
            nsc = to_snake_case(f.name)
            if nsc in fieldset:
                f.rename(idx)
                idx += 1
            fieldset.add(nsc)
        return fields

    @staticmethod
    def group_bitfields(fields):
        grouped_fields = []
        bitfield = []
        bitfield_idx = 0
        for f in fields:
            if f.kind != 'bitfield':
                grouped_fields.append(f)
            else:
                bitfield.append(f)
                if len(bitfield) > 0 and len(bitfield) == bitfield[-1].size:
                    grouped_fields.append(Bitfield(bitfield_idx, bitfield[-1].type, bitfield))
                    bitfield = []
                    bitfield_idx += 1
        return grouped_fields


class Bitfield:
    def __init__(self, idx, dtype, fields):
        self.name = f'bitfield{idx}'
        self.type = dtype
        self.fields = list(enumerate(ParamLayout.dedup_fields(fields)))

    def format(self):
        field_tpl = FIELD_TEMPLATE.format(
            field_name=ParamLayout.fix_name(to_snake_case(self.name)),
            field_type=self.type
        )
        return '\n        '.join(
            '''#[bitflag({flag_name}, {idx})]'''.format(flag_name=ParamLayout.fix_name(flag.name), idx=idx)
            for idx, flag in self.fields
        ) + '\n        ' + field_tpl

    def rename(self, idx):
        self.name = self.name + f'_{idx}'


class Field:
    def_array_re = re.compile(r'(\w+)\s+(\w+)\[(\d+)\]')
    def_bitfield_re = re.compile(r'(\w+)\s+(\w+):(\d+)')
    def_basic_re = re.compile(r'(\w+)\s+(\w+)')

    type_map = {
        's8': 'i8',
        'u8': 'u8',
        's16': 'i16',
        'u16': 'u16',
        's32': 'i32',
        'u32': 'u32',
        'f32': 'f32',
        'fixstr': 'u8',
        'fixstrW': 'u16',
        'dummy8': 'u8',
    }

    def __init__(self, definition):
        if matches := Field.def_array_re.match(definition):
            self.kind = 'array'
            self.name = matches.group(2)
            array_count = int(matches.group(3))
            dtype = Field.type_map.get(matches.group(1))
            self.type = f'[{dtype}; {array_count}]'
        elif matches := Field.def_bitfield_re.match(definition):
            self.kind = 'bitfield'
            self.name = matches.group(2)
            self.type = Field.type_map.get(matches.group(1))
            if self.type == 'u8':
                self.size = 8
            elif self.type == 'u16':
                self.size = 16
            elif self.type == 'u32':
                self.size = 32
            elif self.type == 'u64':
                self.size = 64
            else:
                print(f'[{self.type}]')
        elif matches := Field.def_basic_re.match(definition):
            self.kind = 'normal'
            self.name = matches.group(2)
            self.type = Field.type_map.get(matches.group(1))
        else:
            raise ValueError(f'Couldn\'t parse: {definition}')

    def format(self):
        return FIELD_TEMPLATE.format(
            field_name=ParamLayout.fix_name(to_snake_case(self.name)),
            field_type=self.type
        )

    def rename(self, idx):
        self.name = self.name + f'_{idx}'
            

if __name__ == '__main__':
    layouts = build_param_layouts(sys.argv[1], sys.argv[2])
    print('// **********************************')
    print('// *** AUTOGENERATED, DO NOT EDIT ***')
    print('// **********************************')
    print('use crate::params::*;')
    print('use crate::prelude::*;')
    print('use std::collections::HashMap;')
    print('use std::ffi::c_void;')
    print('use std::lazy::SyncLazy;')
    print('use macro_param::ParamStruct;')

    print('''
unsafe fn get_lambda<T: ParamStruct>() -> BoxedVisitorLambda {
    Box::new(|ptr, v| {
        if let Some(r) = (ptr as *mut T).as_mut() {
            r.visit(&mut *v);
        }
    })
}''')

    print(PARAM_VTABLE_TEMPLATE.format(
        vtable_fields='\n        '.join(
            '''("{param_name}".to_string(), unsafe {{ get_lambda::<{param_name}>() }}),'''
            .format(param_name=l.name)
            for l in layouts
        )
    ), end='')

    for l in layouts:
        print(dedent(l.get_struct()), end='')
