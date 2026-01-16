
module.exports = {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: {
    extend: {
      
      colors: {
        
        ink: {
          50: "#f8fafc",
          100: "#f1f5f9",
          200: "#e2e8f0",
          300: "#cbd5f5", 
          400: "#94a3b8", 
          500: "#64748b",
          600: "#475569",
          700: "#334155", 
          800: "#1e293b", 
          900: "#0f172a"  
        },
        
        mint: {
          400: "#34d399",
          500: "#10b981",
          600: "#059669"
        },
        
        ember: {
          400: "#fb7185",
          500: "#f43f5e",
          600: "#e11d48"
        }
      },
      
      boxShadow: {
        soft: "none",
        glow: "none"
      }
    }
  },
  plugins: []
};