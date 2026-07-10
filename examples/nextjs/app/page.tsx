export default async function Home() {
  let pkg: { name: string; version: string; description: string } | null = null;

  try {
    const res = await fetch('http://localhost:3000/api/v1/packages/react', {
      next: { revalidate: 60 },
    });
    if (res.ok) pkg = await res.json();
  } catch {
    // ignore
  }

  return (
    <div>
      <h1>Dash + Next.js</h1>
      <p>Welcome to the Dash registry Next.js example.</p>
      {pkg ? (
        <div>
          <h2>{pkg.name}</h2>
          <p>{pkg.description}</p>
          <p>Latest version: {pkg.version}</p>
        </div>
      ) : (
        <p>Loading package data...</p>
      )}
    </div>
  );
}
