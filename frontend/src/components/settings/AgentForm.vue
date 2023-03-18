<template>
  <div>
    <p>
      <label for="host">{{ $t("settings.agent.hostLabel") }}</label>
      <input
        class="input input--block"
        type="text"
        v-model="agent.host"
        id="host"
        required="required"
      />
    </p>

    <p>
      <label for="port">{{ $t("settings.agent.port") }}</label>
      <input
        class="input input--block"
        type="number"
        placeholder="7022"
        v-model="agent.port"
        id="port"
        required="required"
      />
    </p>

    <p v-if="isNew">
      <label for="secret">{{ $t("settings.agent.secret") }}</label>
      <input
        class="input input--block"
        type="password"
        id="secret"
        v-model="agent.secret"
        required="required"
      />
    </p>
  </div>
</template>

<script>
export default {
  name: "agent",
  data: () => {
    return {};
  },
  components: {},
  props: ["agent", "isNew"],
  computed: {},
};
</script>
