use practice_tool_core::widgets::label::LabelWidget;
use practice_tool_core::widgets::Widget;

pub(crate) fn label_widget(label: &str) -> Box<dyn Widget> {
    Box::new(LabelWidget::new(label))
}
