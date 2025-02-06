import { notFound } from "next/navigation";
import { mockAPIData } from "@/lib/mock-data";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";

type APIDetailsProps = {
  params: {
    id: string;
  };
};

export default function APIDetails({ params }: APIDetailsProps) {
  const api = mockAPIData.find((api) => api.id === params.id);

  if (!api) {
    notFound();
  }

  return (
    <div className="container mx-auto px-4 py-16">
      <h1 className="text-4xl font-bold mb-8">{api.name}</h1>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <Card className="md:col-span-2">
          <CardHeader>
            <CardTitle>API Description</CardTitle>
          </CardHeader>
          <CardContent>
            <p>{api.fullDescription}</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Pricing</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="mb-4">{api.pricing}</p>
            <Button className="w-full">Subscribe</Button>
          </CardContent>
        </Card>
        <Card className="md:col-span-3">
          <CardHeader>
            <CardTitle>Available Endpoints</CardTitle>
          </CardHeader>
          <CardContent>
            <ul className="space-y-2">
              {api.endpoints.map((endpoint, index) => (
                <li key={index} className="flex items-center">
                  <Badge variant="outline" className="mr-2">
                    {endpoint.method}
                  </Badge>
                  <code className="bg-muted p-1 rounded">{endpoint.path}</code>
                </li>
              ))}
            </ul>
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
