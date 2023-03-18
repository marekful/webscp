import { fetchURL, fetchJSON } from "./utils";

export async function getAll() {
  return fetchJSON(`/api/agents`, {});
}

export async function get(id) {
  return fetchJSON(`/api/agents/${id}`, {});
}

export async function create(agent) {
  const res = await fetchURL(`/api/agents`, {
    method: "POST",
    body: JSON.stringify({
      what: "agent",
      which: [],
      data: agent,
    }),
  });

  if (res.status === 201) {
    return res.headers.get("Location");
  }
}

export async function update(agent, which = ["all"]) {
  await fetchURL(`/api/agents/${agent.id}`, {
    method: "PUT",
    body: JSON.stringify({
      what: "agent",
      which: which,
      data: agent,
    }),
  });
}

export async function remove(id) {
  await fetchURL(`/api/agents/${id}`, {
    method: "DELETE",
  });
}

export async function getVersion(id) {
  return fetchJSON(`/api/agents/${id}/version`, {});
}
