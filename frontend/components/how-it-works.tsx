import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Zap, CreditCard, DollarSign } from "lucide-react";
import Image from "next/image";
import usdcLogo from "../assets/usdc-logo.png";
import usdtLogo from "../assets/usdt-logo.png";

export function HowItWorks() {
  const steps = [
    {
      title: "Streaming payments",
      description: "Pay as you go, real-time access",
      icon: <Zap className="h-8 w-8" />,
    },
    {
      title: "Prepaid credits",
      description: "Deposit once, use off-chain",
      icon: <CreditCard className="h-8 w-8" />,
    },
    {
      title: "One-time payments",
      description: "Simple, on-demand API access",
      icon: <DollarSign className="h-8 w-8" />,
    },
  ];

  return (
    <section className="py-20">
      <h2 className="text-3xl font-bold text-center mb-12">How It Works</h2>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
        {steps.map((step, index) => (
          <Card
            key={index}
            className="bg-card hover:shadow-lg transition-shadow duration-300"
          >
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                {step.icon}
                {step.title}
              </CardTitle>
            </CardHeader>
            <CardContent>
              <CardDescription>{step.description}</CardDescription>
            </CardContent>
          </Card>
        ))}
      </div>
      <div className="mt-12 text-center">
        <p className="text-lg mb-4">Works with any stablecoins</p>
        <div className="flex justify-center space-x-4">
          <Image src={usdcLogo} alt="USDC" width={100} height={100} />

          <Image src={usdtLogo} alt="USDT" width={100} height={100} />
        </div>
      </div>
    </section>
  );
}
