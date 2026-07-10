import { useCallback, useEffect, useState } from 'react';

interface Package {
  name: string;
  description: string;
  version: string;
}

function App() {
  const [pkg, setPkg] = useState<Package | null>(null);

  useEffect(() => {
    fetch('/api/v1/packages/lodash')
      .then((res) => res.json())
      .then((data: Package) => setPkg(data))
      .catch(() => {});
  }, []);

  return (
    <div>
      <h1>Dash + React</h1>
      <p>Welcome to the Dash registry React example.</p>
      {pkg && (
        <div>
          <h2>{pkg.name}</h2>
          <p>{pkg.description}</p>
          <p>Latest version: {pkg.version}</p>
        </div>
      )}
    </div>
  );
}

export default App;
