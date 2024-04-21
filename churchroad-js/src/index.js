const myWorker = new Worker(new URL('./worker.js', import.meta.url));

export function verilogToChurchroad(verilog) {
  return new Promise((resolve, reject) => {
    console.log('resolve' + resolve)
    myWorker.postMessage(verilog);
    myWorker.onmessage = ({ data }) => {
      console.log('resolved to ');
      console.log(data);
      resolve(data);
    };
  });
}