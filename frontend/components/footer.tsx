import { Twitter, Github } from "lucide-react";

export function Footer() {
  return (
    <footer className="py-8 border-t border-border">
      <div className="flex justify-between items-center">
        <div>
          <p className="text-sm text-muted-foreground">
            &copy; 2025 PipeGate. All rights reserved.
          </p>
        </div>
        <div className="flex gap-4">
          Built by Dhruv
          <a
            target="_blank"
            href="https://x.com/0xdhruva"
            className="text-muted-foreground hover:text-foreground"
          >
            <Twitter className="h-5 w-5" />
          </a>
          <a
            target="_blank"
            href="https://github.com/Dhruv-2003/pipegate"
            className="text-muted-foreground hover:text-foreground"
          >
            <Github className="h-5 w-5" />
          </a>
          <a
            target="_blank"
            href="https://github.com/Dhruv-2003/pipegate/blob/main/README.md"
            className="text-sm text-muted-foreground hover:text-foreground"
          >
            Docs
          </a>
        </div>
      </div>
    </footer>
  );
}
