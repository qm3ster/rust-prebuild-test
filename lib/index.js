const { SerialPort } = require("../native")
const { promisify } = require("util")
SerialPort.prototype.write = promisify(SerialPort.prototype.write)
SerialPort.prototype.close = promisify(SerialPort.prototype.close)
const a = cb => {
  const e = new SerialPort("/dev/ttyUSB0", cb, d => console.log("data", d))
  console.log(e)
  let i = 0
  const int = setInterval(async () => {
    e.write(`${i}: ${"|".repeat(i)}\n`)
    i++
  }, 200)
  const int2 = setInterval(async () => {
    console.log("read:", e.poll_read(() => {}))
  }, 100)
  setTimeout(async () => {
    e.close()
  }, 2500)
  setTimeout(() => {
    clearInterval(int)
    clearInterval(int2)
  }, 3000)
}
a(() =>
  setTimeout(() => {
    global.gc()
  }, 100)
)
