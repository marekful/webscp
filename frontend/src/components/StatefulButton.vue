<template>
  <button
    ref="button"
    @click="handle"
    :type="type"
    :class="className"
    :aria-label="$t(labelTr)"
    :title="$t(titleTr)"
  >
    <i v-if="loading" class="material-icons spin">data_usage</i
    >{{ $t(labelTr) }}
  </button>
</template>

<script>
export default {
  name: "stateful-button",
  props: {
    type: {
      type: String,
      default: "button",
      validator: (value) => ["button", "submit"].includes(value),
    },
    handler: {
      type: Function,
      default: () => {},
    },
    waitHandler: {
      type: Boolean,
      default: true,
    },
    className: {
      type: String,
      default: "button button--flat",
    },
    labelTr: {
      type: String,
      default: "",
    },
    titleTr: {
      type: String,
      default: "",
    },
  },
  data: function () {
    return {
      loading: false,
    };
  },
  methods: {
    async handle(event) {
      if (this.loading) return;

      const form = event.target && event.target.form ? event.target.form : null;
      if (form) {
        if (!form.reportValidity()) {
          return;
        }
        if (!this.handler) {
          form.requestSubmit();
        }
      }

      this.loading = true;
      this.$refs.button.setAttribute("disabled", "disabled");
      this.$refs.button.classList.add("loading");

      if (this.handler) {
        if (this.waitHandler) {
          await this.handler(event);
        } else {
          this.handler(event);
        }
      }

      if (this.$refs.button) {
        this.$refs.button.removeAttribute("disabled");
        this.$refs.button.classList.remove("loading");
      }
      this.loading = false;
    },
  },
};
</script>

<style scoped>
button i,
input i {
  font-size: 100%;
  vertical-align: bottom;
  margin-right: 0.5em;
  line-height: 1.2em;
}

button.loading,
input.loading {
  opacity: 0.66;
  cursor: not-allowed;
  background-color: var(--distinct-background);
}
</style>
