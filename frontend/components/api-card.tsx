import Link from "next/link";
import {
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Star } from "lucide-react";

type APICardProps = {
  api: {
    id: string;
    name: string;
    shortDescription: string;
    rating: number;
  };
};

export function APICard({ api }: APICardProps) {
  return (
    <Card className="flex flex-col h-full">
      <CardHeader>
        <CardTitle>{api.name}</CardTitle>
      </CardHeader>
      <CardContent className="flex-grow">
        <p className="text-sm text-muted-foreground mb-4">
          {api.shortDescription}
        </p>
        <div className="flex items-center">
          {[...Array(5)].map((_, i) => (
            <Star
              key={i}
              className={`w-4 h-4 ${
                i < api.rating
                  ? "text-yellow-400 fill-yellow-400"
                  : "text-gray-300"
              }`}
            />
          ))}
          <span className="ml-2 text-sm text-muted-foreground">
            ({api.rating.toFixed(1)})
          </span>
        </div>
      </CardContent>
      <CardFooter>
        <Link href={`/marketplace/${api.id}`} passHref>
          <Button className="w-full">View API</Button>
        </Link>
      </CardFooter>
    </Card>
  );
}
