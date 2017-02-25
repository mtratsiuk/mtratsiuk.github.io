(function () {

  var baseSaturation = 42
  var baseLightness = 42
  var secondaryLightess = 35
  var buttons = document.querySelectorAll('.profile-button')

  function random(from, to) {
    return Math.round((to - from) * Math.random() + from)
  }

  function applyStyle(elements, prop, value) {
    elements.forEach(function (element) {
      element.style[prop] = value
    })
  }

  function hsla(h, s, l, a) {
    return 'hsla(' + h + ',' + s + '%,' + l + '%,' + a + ')'
  }

  function updateColors() {
    var hue = random(0, 360)
    var baseColor = hsla(hue, baseSaturation, baseLightness, 0.9)
    var secondaryColor = hsla(hue, baseSaturation, secondaryLightess, 1)
    applyStyle([document.body], 'backgroundColor', baseColor)
    applyStyle(buttons, 'color', secondaryColor)
  }

  function init() {
    setTimeout(updateColors, 100)
    document.querySelector('.profile-name').addEventListener('click', function () {
      updateColors()
    })
  }

  document.addEventListener("DOMContentLoaded", init)

}())
