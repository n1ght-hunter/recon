use igloo::{bindings::iced::app, widgets::Element as IglooElement};

wasmtime::component::bindgen!({
    path: "../../wit",
    world: "recon-app",
    with: {
        "iced:app": igloo::bindings::iced::app,
        "recon:event-bus": recon_bus::host::recon::event_bus,
    },
    require_store_data_send: true,
});

pub struct ReconState {
    pub wasi: wasmtime_wasi::WasiCtx,
    pub table: wasmtime_wasi::ResourceTable,
    pub bus: recon_bus::Bus,
}

impl ReconState {
    pub fn new(wasi: wasmtime_wasi::WasiCtx, bus: recon_bus::Bus) -> Self {
        Self {
            wasi,
            table: wasmtime_wasi::ResourceTable::new(),
            bus,
        }
    }
}

impl wasmtime_wasi::WasiView for ReconState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl recon_bus::host::EventBusView for ReconState {
    fn event_bus(&mut self) -> recon_bus::host::EventBusCtx<'_> {
        recon_bus::host::EventBusCtx { bus: &self.bus }
    }
}

impl app::text::Host for ReconState {}
impl app::alignment::Host for ReconState {}
impl app::length::Host for ReconState {}
impl app::padding::Host for ReconState {}
impl app::column::Host for ReconState {}
impl app::row::Host for ReconState {}
impl app::container::Host for ReconState {}
impl app::tooltip::Host for ReconState {}
impl app::shared::Host for ReconState {}
impl app::button::Host for ReconState {}
impl app::rule::Host for ReconState {}
impl app::checkbox::Host for ReconState {}
impl app::progress_bar::Host for ReconState {}
impl app::toggler::Host for ReconState {}
impl app::radio::Host for ReconState {}
impl app::table::Host for ReconState {}
impl app::text_input::Host for ReconState {}
impl app::pick_list::Host for ReconState {}
impl app::combo_box::Host for ReconState {}
impl app::float::Host for ReconState {}
impl app::grid::Host for ReconState {}
impl app::image::Host for ReconState {}
impl app::keyed::Host for ReconState {}
impl app::markdown::Host for ReconState {}
impl app::pane_grid::Host for ReconState {}
impl app::slider::Host for ReconState {}
impl app::vertical_slider::Host for ReconState {}
impl app::svg::Host for ReconState {}
impl app::message_types::Host for ReconState {}
impl app::space::Host for ReconState {}
impl app::scrollable::Host for ReconState {}

impl app::message::Host for ReconState {
    fn clone_message(&mut self, message: MessageId) -> MessageId {
        message
    }
}

impl app::shared::HostElement for ReconState {
    fn drop(&mut self, rep: wasmtime::component::Resource<IglooElement>) -> wasmtime::Result<()> {
        self.table.delete(rep)?;
        Ok(())
    }

    fn noop(&mut self, _rep: wasmtime::component::Resource<IglooElement>) {}
}

impl app::element::Host for ReconState {
    fn explain(
        &mut self,
        element: wasmtime::component::Resource<IglooElement>,
        color: app::shared::Color,
    ) -> wasmtime::component::Resource<IglooElement> {
        let element = self.table.delete(element).unwrap();
        self.table
            .push(IglooElement::Explain(Box::new(element), color))
            .unwrap()
    }

    fn text_to_element(
        &mut self,
        text: app::text::Text,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Text(text)).unwrap()
    }

    fn column_to_element(
        &mut self,
        column: app::column::Column,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Column(column)).unwrap()
    }

    fn row_to_element(
        &mut self,
        row: app::row::Row,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Row(row)).unwrap()
    }

    fn container_to_element(
        &mut self,
        container: app::container::Container,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Container(container)).unwrap()
    }

    fn tooltip_to_element(
        &mut self,
        tooltip: app::tooltip::Tooltip,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Tooltip(tooltip)).unwrap()
    }

    fn button_to_element(
        &mut self,
        button: app::button::Button,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Button(button)).unwrap()
    }

    fn rule_to_element(
        &mut self,
        rule: app::rule::Rule,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Rule(rule)).unwrap()
    }

    fn checkbox_to_element(
        &mut self,
        checkbox: app::checkbox::Checkbox,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Checkbox(checkbox)).unwrap()
    }

    fn combo_box_to_element(
        &mut self,
        combo_box: app::combo_box::ComboBox,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::ComboBox(combo_box)).unwrap()
    }

    fn float_to_element(
        &mut self,
        float: app::float::Float,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Float(float)).unwrap()
    }

    fn grid_to_element(
        &mut self,
        grid: app::grid::Grid,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Grid(grid)).unwrap()
    }

    fn image_to_element(
        &mut self,
        image: app::image::Image,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Image(image)).unwrap()
    }

    fn keyed_column_to_element(
        &mut self,
        column: app::keyed::KeyedColumn,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::KeyedColumn(column)).unwrap()
    }

    fn markdown_to_element(
        &mut self,
        markdown: app::markdown::Markdown,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Markdown(markdown)).unwrap()
    }

    fn pane_grid_to_element(
        &mut self,
        grid: app::pane_grid::PaneGrid,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::PaneGrid(grid)).unwrap()
    }

    fn progress_bar_to_element(
        &mut self,
        progress_bar: app::progress_bar::ProgressBar,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table
            .push(IglooElement::ProgressBar(progress_bar))
            .unwrap()
    }

    fn toggler_to_element(
        &mut self,
        toggler: app::toggler::Toggler,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Toggler(toggler)).unwrap()
    }

    fn radio_to_element(
        &mut self,
        radio: app::radio::Radio,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Radio(radio)).unwrap()
    }

    fn pick_list_to_element(
        &mut self,
        pick_list: app::pick_list::PickList,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::PickList(pick_list)).unwrap()
    }

    fn slider_to_element(
        &mut self,
        slider: app::slider::Slider,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Slider(slider)).unwrap()
    }

    fn vertical_slider_to_element(
        &mut self,
        slider: app::vertical_slider::VerticalSlider,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table
            .push(IglooElement::VerticalSlider(slider))
            .unwrap()
    }

    fn svg_to_element(
        &mut self,
        svg: app::svg::Svg,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Svg(svg)).unwrap()
    }

    fn table_to_element(
        &mut self,
        table: app::table::Table,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Table(table)).unwrap()
    }

    fn text_input_to_element(
        &mut self,
        text_input: app::text_input::TextInput,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table
            .push(IglooElement::TextInput(text_input))
            .unwrap()
    }

    fn space_to_element(
        &mut self,
        space: app::space::Space,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table.push(IglooElement::Space(space)).unwrap()
    }

    fn scrollable_to_element(
        &mut self,
        scrollable: app::scrollable::Scrollable,
    ) -> wasmtime::component::Resource<IglooElement> {
        self.table
            .push(IglooElement::Scrollable(scrollable))
            .unwrap()
    }
}
