use practice_tool_core::key::Key;
use practice_tool_core::widgets::group::Group;
use practice_tool_core::widgets::Widget;

pub(crate) fn group(
    label: &str,
    commands: Vec<Box<dyn Widget>>,
    key_close: Key,
) -> Box<dyn Widget> {
    Box::new(Group::new(label, key_close, commands))
}
