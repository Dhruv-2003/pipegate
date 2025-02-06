import { APICard } from "@/components/api-card";
import { mockAPIData } from "@/lib/mock-data";

export default function Marketplace() {
  return (
    <div className="container mx-auto px-4 py-16">
      <h1 className="text-4xl font-bold mb-8">API Marketplace</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {mockAPIData.map((api) => (
          <APICard key={api.id} api={api} />
        ))}
      </div>
    </div>
  );
}
