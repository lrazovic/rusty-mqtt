import { makePost, setActivity } from "./utils.js";

function start() {
  // Create the Sensor object
  let accelerometer = new LinearAccelerationSensor({ frequency: 1 });
  let at = document.getElementById("at").value;
  // The ThingsBoard telemetry URL
  let url = "https://demo.thingsboard.io/api/v1/" + at + "/telemetry";
  // Start the sensor
  accelerometer.start();

  accelerometer.onreading = () => {
    let x = Number(accelerometer.x.toFixed(5));
    let y = Number(accelerometer.y.toFixed(5));
    let z = Number(accelerometer.z.toFixed(5));
    document.getElementById("x").innerHTML = x;
    document.getElementById("y").innerHTML = y;
    document.getElementById("z").innerHTML = z;

    // The JSON objects sent to ThingsBoard
    let telemetry = { x: x, y: y, z: z };
    let activity = setActivity(x, y, z);
    let status = { edgeStatus: activity };

    // Cloud based Model
    makePost(url, JSON.stringify(telemetry));

    // Edge based Model
    makePost(url, JSON.stringify(status));
    document.getElementById("status").innerHTML = activity;
  };

  accelerometer.onerror = (event) => {
    // Handle errors
    console.log(event.error.name, event.error.message);
    document.getElementById("error_name").innerHTML =
      "Error Name: " + event.error.name;
    document.getElementById("error_message").innerHTML =
      "Error Message: " + event.error.message;
  };
}
function stop() {
  // Stop the sensor
  accelerometer.stop();
}

// Link the buttons and the functions
document.getElementById("start").addEventListener("click", start, false);
document.getElementById("stop").addEventListener("click", stop, false);
