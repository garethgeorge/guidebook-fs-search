import test from "node:test";
import * as React from "react";
import _ from "lodash";
import * as api from "../api";

const query = _.debounce(async (query: string) => {
  return api.search(query);
}, 100);

export const App = () => {
  const [searchTerm, setSearchTerm] = React.useState("");
  const [results, setResults] = React.useState<any>();

  React.useEffect(() => {
    (async () => {
      const res = await query(searchTerm);
      setResults(() => res);
    })();
  }, [searchTerm]);

  return (
    <div>
      <h1>Guidebook FS Search</h1>
      <div>
        Search:
        <input
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
        ></input>
      </div>
      <pre>
        <code>{"RESULTS: " + JSON.stringify(results, null, 2)}</code>
      </pre>
    </div>
  );
};
