use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};

pub struct States {
	table: ResourceTable,
	ctx: WasiCtx,
}

impl States {
	pub fn new() -> Self {
		let table = ResourceTable::new();
		let ctx = WasiCtxBuilder::new().build();
		Self { table, ctx }
	}
}

impl WasiView for States {
	fn ctx(&mut self) -> WasiCtxView<'_> {
		WasiCtxView {
			ctx: &mut self.ctx,
			table: &mut self.table,
		}
	}
}
