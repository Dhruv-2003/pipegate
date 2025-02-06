"use client";

import { useState, useEffect } from "react";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { ThemeToggle } from "@/components/theme-toggle";
import { Badge } from "@/components/ui/badge";
import { motion } from "framer-motion";

export function Navbar() {
  const [isScrolled, setIsScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setIsScrolled(window.scrollY > 10);
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 transition-all duration-300 ${
        isScrolled ? "bg-background/80 backdrop-blur-md shadow-md" : ""
      }`}
    >
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          <Link href="/" className="flex items-center space-x-2">
            <svg viewBox="0 0 24 24" className="h-8 w-8" fill="currentColor">
              <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
            </svg>
            <span className="font-bold text-xl">PipeGate</span>
          </Link>
          <div className="flex items-center space-x-4">
            <Link
              href="/marketplace"
              className="text-sm font-medium hover:text-primary transition-colors duration-200 relative group"
            >
              Explore APIs
              <span className="absolute left-0 bottom-0 w-full h-0.5 bg-primary scale-x-0 group-hover:scale-x-100 transition-transform duration-200 origin-left"></span>
            </Link>
            <Link
              href="/dashboard"
              className="text-sm font-medium hover:text-primary transition-colors duration-200 relative group"
            >
              Dashboard
              <Badge variant="secondary" className="ml-2">
                Soon
              </Badge>
              <span className="absolute left-0 bottom-0 w-full h-0.5 bg-primary scale-x-0 group-hover:scale-x-100 transition-transform duration-200 origin-left"></span>
            </Link>
            <ThemeToggle />
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2 }}
            >
              <Button
                size="sm"
                className="bg-primary text-primary-foreground hover:bg-primary/90 transition-all duration-200 transform hover:scale-105"
              >
                <a
                  target="_blank"
                  href="https://github.com/Dhruv-2003/pipegate/blob/main/README.md#how-to-use"
                >
                  Get Started
                </a>
              </Button>
            </motion.div>
          </div>
        </div>
      </div>
    </nav>
  );
}
