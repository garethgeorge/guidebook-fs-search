import axios from "axios";
import qs from "qs";
import { env } from "process";

const backend = env.BACKEND_URL || "";
console.log(`Using backend at ${backend}`);

interface DocumentMetadata {
  path: string;
  size: number;
}

interface Document {
  title: string;
  metadata: DocumentMetadata;
  preview_text: string;
  preview_img_path: string;
}

interface SearchResults {
  results: Document[];
  error: string | undefined;
  latency: number;
}

export const search = async (
  query: string,
  limit: number = 100,
  offset: number = 0
) => {
  const results = await axios.get(`${backend}/query`, {
    params: { query, limit, offset },
  });

  console.log(results);

  return results.data as any as SearchResults;
};
