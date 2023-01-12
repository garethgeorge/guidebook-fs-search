import test from "node:test";
import * as React from "react";
import _ from "lodash";
import * as api from "../api";

let timeout: NodeJS.Timeout | undefined;

export const App = () => {
  const [searchTerm, setSearchTerm] = React.useState("");
  const [results, setResults] = React.useState<any>();

  React.useEffect(() => {
    (async () => {
      if (timeout) {
        clearTimeout(timeout);
      }
      timeout = setTimeout(async () => {
        const res = await api.search(searchTerm);
        setResults(() => res);
      }, 100);
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
