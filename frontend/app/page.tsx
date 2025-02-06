import { Navbar } from "@/components/navbar";
import { HeroSection } from "@/components/hero-section";
import { HowItWorks } from "@/components/how-it-works";
import { WhyUsePipeGate } from "@/components/why-use-pipegate";
import { APIProviders } from "@/components/api-providers";
import { CallToAction } from "@/components/call-to-action";
import { Footer } from "@/components/footer";
import { ThemeProvider } from "@/components/theme-provider";
import { motion } from "framer-motion";
import "@/styles/noise.css";

export default function Home() {
  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem>
      <main className="relative min-h-screen bg-background text-foreground">
        {/* <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 1.3 }}
        > */}
        <div className="noise" />
        <div className="texture" />
        <div className="relative z-10">
          <Navbar />
          <div className="container mx-auto px-4">
            <HeroSection />
            <HowItWorks />
            <WhyUsePipeGate />
            {/* <APIProviders /> */}
            <CallToAction />
            <Footer />
          </div>
        </div>
        {/* </motion.div> */}
      </main>
    </ThemeProvider>
  );
}
