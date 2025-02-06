"use client"

import { useState } from "react"
import { motion } from "framer-motion"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"

export function APIProviders() {
  const [flipped, setFlipped] = useState(false)

  const benefits = [
    "Lower fees compared to traditional payment processors",
    "No API key management required",
    "Instant onboarding for your customers",
    "Flexible payment models (streaming, prepaid, one-time)",
  ]

  return (
    <section id="explore-apis" className="py-20">
      <h2 className="text-3xl font-bold text-center mb-12">Why integrate PipeGate?</h2>
      <div className="flex justify-center">
        <div className="relative w-96 h-64 cursor-pointer" onClick={() => setFlipped(!flipped)}>
          <motion.div
            className="absolute w-full h-full"
            initial={false}
            animate={{ rotateY: flipped ? 180 : 0 }}
            transition={{ duration: 0.6 }}
            style={{ backfaceVisibility: "hidden" }}
          >
            <Card className="w-full h-full flex flex-col justify-center items-center bg-card hover:shadow-lg transition-shadow duration-300">
              <CardHeader>
                <CardTitle>API Providers</CardTitle>
              </CardHeader>
              <CardContent>
                <CardDescription>Click to see the benefits</CardDescription>
              </CardContent>
            </Card>
          </motion.div>
          <motion.div
            className="absolute w-full h-full"
            initial={{ rotateY: 180 }}
            animate={{ rotateY: flipped ? 0 : -180 }}
            transition={{ duration: 0.6 }}
            style={{ backfaceVisibility: "hidden" }}
          >
            <Card className="w-full h-full flex flex-col justify-center items-center bg-card hover:shadow-lg transition-shadow duration-300">
              <CardHeader>
                <CardTitle>Benefits</CardTitle>
              </CardHeader>
              <CardContent>
                <ul className="list-disc list-inside">
                  {benefits.map((benefit, index) => (
                    <li key={index} className="text-sm mb-2">
                      {benefit}
                    </li>
                  ))}
                </ul>
              </CardContent>
            </Card>
          </motion.div>
        </div>
      </div>
    </section>
  )
}

