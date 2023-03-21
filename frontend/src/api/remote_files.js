import { fetchURL, removePrefix } from "./utils";

export async function fetch(agent_id, url) {
  console.log("remote_files.fetch() > ", agent_id, url);

  url = removePrefix(url);

  const res = await fetchURL(
    `/api/remote/resources/${agent_id}/${encodeURIComponent(url)}`,
    {}
  );

  let data = await res.json();

  if (data.error) {
    let e = JSON.parse(data.error);
    throw new Error(e.error);
  }

  data = JSON.parse(data.resource);

  data.url = `/files${url}`;

  if (data.isDir) {
    if (!data.url.endsWith("/")) data.url += "/";
    data.items = data.items.map((item, index) => {
      item.index = index;
      item.url = `${data.url}${encodeURIComponent(item.name)}`;

      if (item.isDir) {
        item.url += "/";
      }

      return item;
    });
  }

  return data;
}
