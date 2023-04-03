<template>
  <div class="card transfers">
    <section class="card-content">
      <div v-for="(transfer, index) in transfers" :key="index" class="transfer">
        <div class="title ellipse" :title="transfer.title">
          <i class="material-icons">content_copy_outlined</i>
          {{ transfer.title }}
        </div>
        <div class="content">
          <div
            :class="transfer.cancelable ? 'remove' : 'remove disabled'"
            @click="handleCancel"
            :data-id="transfer.transferID"
            :title="
              transfer.pending ? $t('transfer.cancel') : $t('transfer.remove')
            "
          >
            <i class="material-icons">
              {{ transfer.cancelable ? "cancel" : "highlight_off" }}
            </i>
          </div>
          <div
            :class="
              'icon ' +
              (transfer.error === true || transfer.canceled === true
                ? 'icon-error'
                : transfer.pending === false
                ? 'icon-success'
                : '')
            "
          >
            <i class="material-icons">{{ transfer.icon }}</i>
          </div>
          <div v-if="transfer.error !== true" class="status">
            <span>
              {{ transfer.status }}
              <span
                v-if="transfer.stats && transfer.stats.progress.length > 0"
                class="stats"
              >
                <span>{{ transfer.stats.progress[0] }}.</span>
                <small>
                  {{ transfer.stats.progress[1] }}
                  {{ transfer.stats.progress[2] }}
                </small>
                <span>of {{ transfer.stats.total[0] }}.</span>
                <small>
                  {{ transfer.stats.total[1] }}{{ transfer.stats.total[2] }}
                </small>
              </span>
            </span>
          </div>
          <div v-else class="error">
            <span>{{ transfer.status }}</span>
          </div>
          <div class="details">
            <div class="icon" @click="showDetails(transfer)">
              <i class="material-icons">arrow_drop_down_circle</i>
              <span class="label">{{ $t("transfer.showDetails") }}</span>
            </div>
            <div v-if="transfer.showDetails === true" class="content">
              <div v-for="(item, index) in transfer.items" :key="index">
                <span>{{ item.from }}</span>
                <span>{{ item.to }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div v-if="transfers.length === 0" class="transfer no-content">
        No transfers
      </div>
    </section>

    <section class="card-action">
      <button
        class="button button--flat"
        @click="$store.commit('closeHovers')"
        :aria-label="$t('buttons.cancel')"
        :title="$t('buttons.cancel')"
      >
        {{ $t("buttons.close") }}
      </button>
    </section>
  </div>
</template>

<script>
import { mapState } from "vuex";
import transfers from "@/utils/transfers";
import { remote_files } from "@/api";

export default {
  name: "transfers",
  data: function () {
    return {};
  },
  computed: {
    ...mapState(["req", "transfers"]),
  },
  methods: {
    cancel: function () {
      this.$store.commit("closeHovers");
    },
    handleCancel: function (event) {
      let transferID = event.target.parentNode.dataset["id"];
      let transfer = transfers.get(this.transfers, transferID);

      if (!transfer || !transfer.cancelable) {
        return;
      }

      if (!transfer.pending) {
        transfers.remove(this.$store, transferID);
        setTimeout(() => {
          transfers.setButtonActive(this.transfers);
        }, 100);

        return;
      }

      transfer.sseClient && transfer.sseClient.close();

      remote_files
        .cancelTransfer(transfer.agent.id, transferID)
        .then(() => {
          transfers.update(this.$store, {
            transferID,
            canceled: true,
            pending: false,
            icon: "highlight_off",
            status: "Canceled",
          });
        })
        .catch(() => {
          transfers.remove(this.$store, transferID);
        })
        .finally(() => {
          setTimeout(() => {
            transfers.setButtonActive(this.transfers);
          }, 100);
        });
    },
    showDetails: function (transfer) {
      transfers.update(this.$store, {
        transferID: transfer.transferID,
        showDetails: !transfer.showDetails,
      });
    },
  },
};
</script>

<style>
.card.transfers {
  position: fixed;
  top: 4.2em;
  right: 5%;
  z-index: 99999;
  color: #6f6f6f;
  max-width: 30em;
  width: 90%;
  max-height: 95%;
  animation: 0.1s show forwards;
}
.transfer {
  border-width: 0 0 1px 0;
  border-style: solid;
  border-color: #ddd;
  min-height: 5em;
}

.transfer:first-child {
  border-width: 1px 0;
}

.ellipse {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.transfer > .title {
  font-size: 110%;
  padding: 0.75em 1em;
  border-bottom: 1px solid #eee;
  background-color: #fdfdfd;
  cursor: default;
}

.transfer > .content {
  padding: 1em;
}

.transfer > .title > i {
  font-size: 1em;
  max-width: 1em;
  margin: 0 0.4em 0 0;
  color: var(--dark-grey);
  vertical-align: middle;
  opacity: 0.8;
}

.transfer .error,
.transfer .status {
  display: table;
}

.transfer .status {
  line-height: 2.5em;
  font-size: 105%;
  text-transform: capitalize;
}

.transfer .error {
  color: var(--dark-red);
}

.transfer .stats {
  margin: 0 0 0 0.5em;
  font-weight: bold;
}

.transfer .stats small {
  margin: 0 0.5em 0 0;
}

.transfer > .content > .icon,
.transfer > .content > .icon-error {
  float: left;
  max-width: 2em;
  margin: 0 0.75em 0 0;
  padding: 0.25em 0 0 0;
}

.transfer > .content > .icon i {
  font-size: 2rem;
}

.transfer > .content > .icon:not(.icon-error):not(.icon-success) i {
  color: #546e7a;
  animation-duration: 5s;
  animation-name: change-color;
  animation-iteration-count: infinite;
  animation-direction: alternate;
}

@keyframes change-color {
  from {
    color: #546e7a;
  }

  to {
    color: var(--icon-blue);
  }
}

.transfer .icon-error i {
  color: var(--dark-red);
}

.transfer .icon-success i {
  color: var(--icon-green);
}

.transfer.no-content {
  color: var(--mid-grey);
  padding: 2em;
}

.transfer > .content > .remove {
  float: right;
  margin: 0 0 0 0.5em;
  padding: 1em 0 0 0;
  opacity: 0.4;
  cursor: pointer;
}

.transfer .remove:hover {
  opacity: 0.8;
}

.transfer .remove.disabled {
  opacity: 0.3;
  cursor: default;
}

.transfer > .content > .details {
  margin-top: 0.25em;
}

.transfer > .content > .details > .icon {
  text-align: center;
  margin-bottom: -0.5em;
  cursor: pointer;
  opacity: 0.5;
}

.transfer > .content > .details > .icon:hover {
  opacity: 0.8;
}

.transfer > .content > .details > .icon > i {
  font-size: 1em;
  vertical-align: middle;
}

.transfer > .content > .details > .icon > .label {
  font-size: 90%;
  margin: 0 0 0 0.25em;
}

.transfer > .content > .details > .content {
  margin-top: 1em;
}

section.card-action {
  padding: 0.5em;
}
</style>
