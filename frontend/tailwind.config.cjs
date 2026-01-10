module.exports = {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      fontFamily: {
        display: ["Space Grotesk", "Segoe UI", "sans-serif"],
        mono: ["IBM Plex Mono", "Consolas", "monospace"]
      },
      colors: {
        ink: {
          50: "#f8fafc",
          100: "#e2e8f0",
          200: "#cbd5f5",
          300: "#94a3b8",
          400: "#64748b",
          500: "#475569",
          600: "#334155",
          700: "#1e293b",
          800: "#0f172a",
          900: "#0b1220"
        },
        mint: {
          400: "#3dd6b0",
          500: "#12b981",
          600: "#0f8f66"
        },
        ember: {
          400: "#f9735b",
          500: "#ef4444",
          600: "#dc2626"
        }
      },
      boxShadow: {
        soft: "0 18px 40px -24px rgba(15, 23, 42, 0.65)",
        glow: "0 0 40px rgba(61, 214, 176, 0.25)"
      }
    }
  },
  plugins: []
};
