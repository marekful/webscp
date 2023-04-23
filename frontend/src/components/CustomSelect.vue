<template>
  <div class="custom-select" :tabindex="tabindex" @blur="open = false">
    <div class="selected" :class="{ open: open }" @click="open = !open">
      {{ selected }}
    </div>
    <div class="items" :class="{ selectHide: !open }">
      <div
        v-for="(option, i) of options"
        :key="i"
        :class="'option-' + i + (i === selectedIndex ? ' selected' : '')"
        @click="click($event, option, i)"
      >
        {{ option }}
      </div>
    </div>
  </div>
</template>

<script>
export default {
  name: "custom-select",
  props: {
    options: {
      type: Array,
      required: true,
    },
    default: {
      type: String,
      required: false,
      default: null,
    },
    tabindex: {
      type: Number,
      required: false,
      default: 0,
    },
  },
  data() {
    return {
      selected: this.default
        ? this.default
        : this.options.length > 0
        ? this.options[0]
        : null,
      open: false,
      selectedIndex: 0,
    };
  },
  watch: {
    tabindex(index) {
      let option = this.options[index];
      this.selected = option;
      this.$emit("input", { option, index });
      this.setSelected(index);
    },
  },
  methods: {
    click: function (event, option, index) {
      this.selected = option;
      this.selectedIndex = index;
      this.open = false;
      this.$emit("input", { option, index });
      this.setSelected(index);
    },
    setSelected(index) {
      let toSelect = this.$el.querySelector(`.items > div.option-${index}`);
      let selected = this.$el.querySelector(`.items > div.selected`);
      selected.classList.remove("selected");
      toSelect.classList.add("selected");
    },
  },
};
</script>

<style scoped>
.custom-select {
  position: relative;
  width: 100%;
  text-align: left;
  outline: none;
  line-height: 2.25em;
}

.custom-select > .selected {
  border-radius: 6px;
  padding-left: 1em;
  cursor: pointer;
  user-select: none;
}

.custom-select > .selected:after {
  position: absolute;
  content: "";
  top: 1em;
  right: 1em;
  width: 0;
  height: 0;
}

.custom-select .items {
  position: absolute;
  overflow: hidden;
  left: 0;
  right: 0;
  z-index: 1;
}

.custom-select .items div {
  padding-left: 1em;
  cursor: pointer;
  user-select: none;
}

.selectHide {
  display: none;
}
</style>
