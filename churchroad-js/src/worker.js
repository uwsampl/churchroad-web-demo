import { runYosys } from '@yowasp/yosys';

await runYosys();

async function work() {
  // Set callback to handle messages passed to the worker.
  self.onmessage = async ({ data: data }) => {
    let outFiles = runYosys(
      ["-q", "-p", "read_verilog test.v; prep; write_lakeroad test.egg"],
      { "test.v": data },
      { synchronously: true });
    self.postMessage(outFiles['test.egg']);
  };
}

work()
