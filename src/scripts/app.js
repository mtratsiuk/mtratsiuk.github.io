;(function() {
  document.addEventListener('DOMContentLoaded', init)

  var easterEggs = [
    { predicate: isFourteenth, action: applyFourteenthEgg },
    { predicate: T, action: applyDefaultEgg },
  ]

  function init() {
    setTimeout(updateColors, 100)

    var action = (
      easterEggs.find(function(egg) {
        return egg.predicate && egg.predicate()
      }) || {}
    ).action

    document
      .querySelector('.profile-name')
      .addEventListener('click', function() {
        action && action()
      })
  }

  function random(from, to) {
    return Math.round((to - from) * Math.random() + from)
  }

  function applyStyle(elements, prop, value) {
    elements.forEach(function(element) {
      element.style[prop] = value
    })
  }

  function hsla(h, s, l, a) {
    return 'hsla(' + h + ',' + s + '%,' + l + '%,' + a + ')'
  }

  function updateColors() {
    var baseSaturation = 42
    var baseLightness = 42
    var secondaryLightess = 35
    var buttons = document.querySelectorAll('.profile-button')

    var hue = random(0, 360)
    var baseColor = hsla(hue, baseSaturation, baseLightness, 0.9)
    var secondaryColor = hsla(hue, baseSaturation, secondaryLightess, 1)
    applyStyle([document.body], 'backgroundColor', baseColor)
    applyStyle(buttons, 'color', secondaryColor)
  }

  function applyDefaultEgg() {
    return updateColors()
  }

  function applyFourteenthEgg() {
    var container = document.querySelector('.profile')

    applyStyle([document.body], 'backgroundColor', '#0f4c81')
    container.classList.add('egg-frt')
    container.innerHTML =
      '<div class="frt"><div class="frt-heart"></div><div class="frt-note">#0f4c81 Classic Blue</div></div>'
  }

  function isFourteenth() {
    var date = new Date()
    return date.getDate() === 14 && date.getMonth() === 1
  }

  function T() {
    return true
  }
})()
