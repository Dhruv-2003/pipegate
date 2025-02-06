import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { CheckCircle } from "lucide-react";

export function WhyUsePipeGate() {
  const reasons = [
    {
      title: "Stripe Alternative for API monetisation",
      description: "No 3% transaction fee.",
    },
    {
      title: "Alternative to traiditional API keys",
      description: "No API keys, just wallets.",
    },
    {
      title: "Self served onboarding",
      description: "No backend infra required anymore.",
    },
  ];

  return (
    <section className="py-20">
      <h2 className="text-3xl font-bold text-center mb-12">
        Why Use PipeGate?
      </h2>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
        {reasons.map((reason, index) => (
          <Card key={index} className="bg-card">
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <CheckCircle className="h-6 w-6 text-green-500" />
                {reason.title}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <CardDescription>{reason.description}</CardDescription>
            </CardContent>
          </Card>
        ))}
      </div>
    </section>
  );
}
