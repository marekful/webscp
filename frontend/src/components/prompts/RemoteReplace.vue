<template>
  <div class="card floating">
    <div class="card-title">
      <h2>{{ $t("prompts.agent.replace") }}</h2>
    </div>

    <div class="card-content">
      <p>{{ $t("prompts.agent.replaceMessage") }}</p>
    </div>

    <div class="card-action">
      <button
        class="button button--flat button--grey"
        @click="$store.commit('closeHovers')"
        :aria-label="$t('buttons.cancel')"
        :title="$t('buttons.cancel')"
      >
        {{ $t("buttons.cancel") }}
      </button>
      <button
        class="button button--flat button--blue"
        @click="(event) => showConfirm(event, 'keep')"
        :aria-label="$t('buttons.keep')"
        :title="$t('buttons.keep')"
      >
        {{ $t("buttons.keep") }}
      </button>
      <button
        class="button button--flat button--red"
        @click="(event) => showConfirm(event, 'overwrite')"
        :aria-label="$t('buttons.replace')"
        :title="$t('buttons.replace')"
      >
        {{ $t("buttons.replace") }}
      </button>
    </div>
  </div>
</template>

<script>
import { mapState } from "vuex";

export default {
  name: "remote-replace",
  computed: mapState(["showConfirm"]),
};
</script>
