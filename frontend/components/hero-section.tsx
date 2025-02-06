"use client";

import { motion } from "framer-motion";
import { Button } from "@/components/ui/button";
import { ArrowRight } from "lucide-react";

export function HeroSection() {
  return (
    <section className="py-20 mt-16 text-center relative overflow-hidden">
      <motion.h1
        className="text-4xl md:text-6xl font-bold mb-4"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        Monetize APIs without API keys or hefty fees
      </motion.h1>
      <motion.p
        className="text-xl md:text-2xl mb-8 text-muted-foreground"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.2 }}
      >
        Stream payments, use prepaid credits, or charge per call—all without
        managing API keys.
      </motion.p>
      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5, delay: 0.4 }}
      >
        <Button
          size="lg"
          className="text-lg bg-primary text-primary-foreground hover:bg-primary/90 transition-all duration-200 transform hover:scale-105"
        >
          <a
            target="_blank"
            href="https://github.com/Dhruv-2003/pipegate/blob/main/README.md#how-to-use"
          >
            Get Started
          </a>
          <ArrowRight className="ml-2 h-5 w-5" />
        </Button>
      </motion.div>
      <motion.div
        className="mt-16"
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 1, delay: 0.6 }}
      >
        <APIFlowAnimation />
      </motion.div>
    </section>
  );
}

function APIFlowAnimation() {
  return (
    <div className="relative h-64 w-full">
      <svg
        width="100%"
        height="100%"
        viewBox="0 0 800 200"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="rgba(59, 130, 246, 0.5)" />
            <stop offset="100%" stopColor="rgba(147, 51, 234, 0.5)" />
          </linearGradient>
        </defs>
        <motion.path
          d="M0 100 C 200 50, 600 150, 800 100"
          stroke="url(#gradient)"
          strokeWidth="4"
          fill="none"
          initial={{ pathLength: 0 }}
          animate={{ pathLength: 1 }}
          transition={{
            duration: 2,
            repeat: Number.POSITIVE_INFINITY,
            ease: "linear",
          }}
        />
      </svg>
      <APINode x={100} y={100} delay={0} />
      <APINode x={300} y={50} delay={0.5} />
      <APINode x={500} y={150} delay={1} />
      <APINode x={700} y={100} delay={1.5} />
    </div>
  );
}

type APINodeProps = {
  x: number;
  y: number;
  delay: number;
};

function APINode({ x, y, delay }: APINodeProps) {
  return (
    <motion.div
      className="absolute w-4 h-4 bg-primary rounded-full shadow-lg"
      style={{ left: x, top: y }}
      initial={{ scale: 0, opacity: 0 }}
      animate={{ scale: [0, 1.2, 1], opacity: [0, 1, 0] }}
      transition={{ duration: 2, repeat: Number.POSITIVE_INFINITY, delay }}
    />
  );
}
