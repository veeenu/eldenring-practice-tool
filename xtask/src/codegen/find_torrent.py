# Quick and dirty script to find out Torrent's position

import psutil
import ctypes
from ctypes import wintypes


def find_process_by_name(process_name):
    # Iterate through all running processes
    for proc in psutil.process_iter():
        if proc.name() == process_name.lower():
            return ctypes.windll.kernel32.OpenProcess(0x10, False, proc.pid)
    return None


def get_module_handle(h_process, module_name):
    cb_needed = wintypes.DWORD()
    h_module = wintypes.HMODULE()
    ctypes.windll.psapi.EnumProcessModulesEx(h_process, ctypes.byref(h_module),
                                             ctypes.sizeof(h_module),
                                             ctypes.byref(cb_needed),
                                             wintypes.DWORD(0x03))
    num_modules = cb_needed.value // ctypes.sizeof(h_module)
    module_handles = (wintypes.HMODULE * num_modules)()
    ctypes.windll.psapi.EnumProcessModulesEx(h_process, ctypes.byref(
        module_handles), cb_needed, ctypes.byref(cb_needed),
        ctypes.wintypes.DWORD(0x03))

    for h_module in module_handles:
        module_name_buf = ctypes.create_string_buffer(1024)
        ctypes.windll.psapi.GetModuleFileNameExA(
            h_process, h_module, module_name_buf,
            ctypes.sizeof(module_name_buf)
        )
        module_name_from_path = module_name_buf.value.split("\\")[-1]
        print(module_name_from_path)
        if module_name_from_path.lower() == module_name.lower():
            return h_module


def get_base(h_process):
    module_info = wintypes.MODULEINFO()
    h_module = get_module_handle(h_process, 'eldenring.exe')
    ctypes.windll.psapi.GetModuleInformation(
        h_process, h_module, ctypes.byref(module_info),
        ctypes.sizeof(module_info))
    print(module_info)


def read_pointer_chain(h_process, base_address, offsets):
    # Allocate memory for the buffer
    buffer_size = ctypes.sizeof(ctypes.c_void_p)
    buffer = ctypes.c_void_p()
    print('Reading [' + hex(base_address) + ', '.join(map(hex, offsets)) + ']')

    # Read the initial address
    ctypes.windll.kernel32.ReadProcessMemory(
        h_process, ctypes.c_void_p(base_address),
        ctypes.byref(buffer), buffer_size, None)
    address = ctypes.c_void_p.from_buffer(buffer).value

    # Follow the chain of offsets
    for offset in offsets:
        print(f'  ({hex(address)}, {hex(offset)})')
        # Read the next address in the chain
        ctypes.windll.kernel32.ReadProcessMemory(
            h_process, ctypes.c_void_p(address + offset), ctypes.byref(buffer),
            buffer_size, None)
        address = ctypes.c_void_p.from_buffer(buffer).value

    return address


def read_u32(h_process, address):
    buffer_size = ctypes.sizeof(ctypes.c_uint32)
    buffer = ctypes.c_uint32()

    ctypes.windll.kernel32.ReadProcessMemory(
        h_process, ctypes.c_void_p(address),
        ctypes.byref(buffer), buffer_size, None)
    return buffer.value


# Find the process by name
process = find_process_by_name('eldenring.exe')
base = 0x7FF66BBE0000
world_chr_man = 0x3cd9998
player_ins = 0x1E508
chr_set = 0x1DED8
pg_data_offs = 0x580
torrent_id_offs = 0x930

if process:
    # Read the chain of pointers starting at base_address and following the offsets
    torrent_id_addr = read_pointer_chain(process, base + world_chr_man, [
        player_ins, pg_data_offs
    ])
    torrent_id = read_u32(process, torrent_id_addr + torrent_id_offs)
    print(f'Torrent ID: {hex(torrent_id)}')
    torrent_off = (torrent_id & 0x0FF00000) >> 20
    print(f'Torrent offs: {hex(torrent_off)}')
    enemy_ins = read_pointer_chain(process, base + world_chr_man, [
        chr_set + torrent_off * 8
    ])
    print(hex(enemy_ins))
else:
    print("Process not found")
