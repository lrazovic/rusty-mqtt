export const makePost = function (url, body) {
  fetch(url, {
    method: "POST",
    mode: "no-cors",
    headers: {
      "Content-type": "application/json",
    },
    body: body,
  }).catch(function (error) {
    console.log("Request failed", error);
  });
};

export const setActivity = function (x, y, z) {
  let norm = Math.sqrt(x * x + y * y + z * z);
  if (norm > 0.5) return "Moving";
  else return "Still";
};

const arrayToMatrix = function (array, n) {
  return array.reduce(
    (rows, key, index) =>
      (index % n == 0 ? rows.push([key]) : rows[rows.length - 1].push(key)) &&
      rows,
    []
  );
};
const rotationMatrix = function (array) {
  let x = array.map(function (val) {
    return val.slice(0, -1);
  });
  x.pop();
  return x;
};
