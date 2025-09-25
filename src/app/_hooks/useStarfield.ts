"use client";

import { useEffect, useState } from "react";

export function useStarfield() {
  const [isDarkMode, setIsDarkMode] = useState(false);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    // Mark as mounted to avoid hydration mismatch
    setMounted(true);
    
    // Randomize star positions on each page load
    const randomizeStars = () => {
      const root = document.documentElement;
      
      // Generate completely random star patterns for each layer
      const generateRandomStars = (count: number, maxSize: number = 2) => {
        const stars = [];
        for (let i = 0; i < count; i++) {
          const x = Math.random() * 100; // 0-100%
          const y = Math.random() * 100; // 0-100%
          const size = Math.random() * maxSize + 0.5; // 0.5-2.5px
          stars.push({ position: `${x}% ${y}%`, size });
        }
        return stars;
      };
      
      // Generate random positions for each layer with different sizes
      const layer1Stars = generateRandomStars(80, 1);
      const layer2Stars = generateRandomStars(60, 1.5);
      const layer3Stars = generateRandomStars(40, 2);
      
      // Create CSS for each layer with random positions and sizes
      const layer1CSS = layer1Stars.map((star, i) => {
        const colors = ['#9bb0ff', '#aabfff', '#cad7ff', '#f8f7ff', '#fff4ea'];
        const color = colors[i % colors.length];
        const size = `${star.size}px ${star.size}px`;
        return `radial-gradient(${size} at ${star.position}, ${color}, transparent)`;
      }).join(', ');
      
      const layer2CSS = layer2Stars.map((star, i) => {
        const colors = ['#ffcc6f', '#ffad51', '#9bb0ff', '#cad7ff'];
        const color = colors[i % colors.length];
        const size = `${star.size}px ${star.size}px`;
        return `radial-gradient(${size} at ${star.position}, ${color}, transparent)`;
      }).join(', ');
      
      const layer3CSS = layer3Stars.map((star, i) => {
        const colors = ['#fff4ea', '#aabfff', '#ffcc6f'];
        const color = colors[i % colors.length];
        const size = `${star.size}px ${star.size}px`;
        return `radial-gradient(${size} at ${star.position}, ${color}, transparent)`;
      }).join(', ');
      
      // Apply the random star patterns
      root.style.setProperty('--starfield-layer1-bg', layer1CSS);
      root.style.setProperty('--starfield-layer2-bg', layer2CSS);
      root.style.setProperty('--starfield-layer3-bg', layer3CSS);
    };
    
    randomizeStars();
    
    // Check initial theme
    const checkTheme = () => {
      setIsDarkMode(document.documentElement.classList.contains('dark'));
    };
    
    checkTheme();
    
    // Watch for theme changes with throttling
    let timeoutId: NodeJS.Timeout;
    const observer = new MutationObserver(() => {
      clearTimeout(timeoutId);
      timeoutId = setTimeout(checkTheme, 50); // Throttle to reduce updates
    });
    
    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['class'],
    });

    return () => {
      observer.disconnect();
      clearTimeout(timeoutId);
    };
  }, []);

  return { isDarkMode, mounted };
}
