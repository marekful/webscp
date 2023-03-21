<template>
  <div class="custom-select" :tabindex="tabindex" @blur="open = false">
    <div class="selected" :class="{ open: open }" @click="open = !open">
      {{ selected }}
    </div>
    <div class="items" :class="{ selectHide: !open }">
      <div
        v-for="(option, i) of options"
        :key="i"
        @click="click($event, option)"
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
    };
  },
  mounted() {
    this.$emit("input", this.selected);
  },
  methods: {
    click: function(event, option) {
      this.selected = option;
      this.open = false;
      this.$emit('input', option);
    },
  }
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

.custom-select .selected {
  border-radius: 6px;
  border: 1px solid var(--dark-blue);
  background-color: var(--moon-grey);
  color: #000;
  padding-left: 1em;
  cursor: pointer;
  user-select: none;
}

.custom-select .selected.open {
  border: 1px solid var(--dark-blue);
  border-radius: 6px 6px 0 0;
}

.custom-select .selected:after {
  position: absolute;
  content: "";
  top: 1em;
  right: 1em;
  width: 0;
  height: 0;
  border: 5px solid transparent;
  border-color: var(--dark-blue) transparent transparent transparent;
}

.custom-select .items {
  color: #000;
  border-radius: 0px 0px 6px 6px;
  overflow: hidden;
  border-right: 1px solid var(--dark-blue);
  border-left: 1px solid var(--dark-blue);
  border-bottom: 1px solid var(--dark-blue);
  position: absolute;
  background-color: var(--moon-grey);
  left: 0;
  right: 0;
  z-index: 1;
}

.custom-select .items div {
  color: #000;
  padding-left: 1em;
  cursor: pointer;
  user-select: none;
}

.custom-select .items div:hover {
  background-color: #fff;
  color: var(--dark-blue);
}

.selectHide {
  display: none;
}
</style>
