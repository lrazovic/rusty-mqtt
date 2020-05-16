import { makePost, setActivity } from "./utils.js";

let accelerometer = new LinearAccelerationSensor({ frequency: 1 });
//const array = new Float32Array(16);
// /const options = { frequency: 1, referenceFrame: "device" };
//const sensor = new AbsoluteOrientationSensor(options);

function start() {
  let at = document.getElementById("at").value;
  let url = "https://demo.thingsboard.io/api/v1/" + at + "/telemetry";
  accelerometer.start();

  accelerometer.onreading = () => {
    let x = Number(accelerometer.x.toFixed(5));
    let y = Number(accelerometer.y.toFixed(5));
    let z = Number(accelerometer.z.toFixed(5));
    document.getElementById("x").innerHTML = x;
    document.getElementById("y").innerHTML = y;
    document.getElementById("z").innerHTML = z;
    let telemetry = { x: x, y: y, z: z };
    let activity = setActivity(x, y, x);
    let status = { status: activity };
    // Cloud based Model
    makePost(url, JSON.stringify(telemetry));

    // Edge based Model
    makePost(url, JSON.stringify(status));
    document.getElementById("status").innerHTML = activity;
  };

  accelerometer.onerror = (event) => {
    console.log(event.error.name, event.error.message);
    document.getElementById("error_name").innerHTML =
      "Error Name: " + event.error.name;
    document.getElementById("error_message").innerHTML =
      "Error Message: " + event.error.message;
  };
}
function stop() {
  accelerometer.stop();
}

document.getElementById("start").addEventListener("click", start, false);
document.getElementById("stop").addEventListener("click", stop, false);
