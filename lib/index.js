const { SerialPort } = require("../native")
const { promisify } = require("util")
SerialPort.prototype.write = promisify(SerialPort.prototype.write)
SerialPort.prototype.close = promisify(SerialPort.prototype.close)
const a = cb => {
  const e = new SerialPort("/dev/ttyUSB0", cb)
  console.log(e)
  let i = 0
  const int = setInterval(async () => {
    await e.write(`write ${i}: ${"|".repeat(i)}\n`)
    // console.log("[js] wrote", i)
    i++
  }, 5)
  setTimeout(async () => {
    await e.close()
  }, 2000)
  setTimeout(() => {
    clearInterval(int)
  }, 3000)
}
a(() =>
  setTimeout(() => {
    global.gc()
  }, 100)
)
