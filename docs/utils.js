export const thinhsboardPost = function (url, body) {
  fetch(url, {
    method: "POST",
    mode: "no-cors",
    headers: {
      "Content-type": "application/json",
    },
    body: body,
  })
    .then((response) => {
      console.log(response);
    })
    .catch(function (error) {
      console.log("Request failed", error);
    });
};

export const localPost = function (body) {
  fetch("/", {
    method: "POST",
    mode: "no-cors",
    headers: {
      "Content-type": "application/json",
    },
    body: body,
  })
    .then((response) => {
      return response;
    })
    .catch(function (error) {
      console.log("Request failed", error);
    });
};

export const setActivity = function (array, x, y, z) {
  let mat = arrayToMatrix(array, 4);
  let rotMatrix = rotationMatrix(mat);
  let velocity = [x, y, z];
  let final = math.multiply(rotMatrix, velocity);
  return final;
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
