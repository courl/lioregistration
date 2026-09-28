/** @type {import('stylelint').Config} */
export default {
  extends: [
    "stylelint-config-standard-scss", // CSS + SCSS rules
    "stylelint-config-recommended-vue/scss", // parses <style> blocks in SFCs
  ],
  rules: {
    "declaration-no-important": true,
    "selector-max-id": 0,
    "selector-max-specificity": "0,4,2",
    "max-nesting-depth": 3,
    "no-descending-specificity": true,
  },
}
