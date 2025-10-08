'use client';

import { useStarfield } from '@/app/_hooks/useStarfield';

type PlanetConfig = {
  name: string;
  diameterPx: number;
  sizePx: number;
  period: string;
  delay: string;
  e?: number;
};

const PLANETS: PlanetConfig[] = [
  {
    name: 'mercury',
    diameterPx: 150,
    sizePx: 4,
    period: '6s',
    delay: '-2s',
    e: 0.7,
  },
  {
    name: 'venus',
    diameterPx: 200,
    sizePx: 5,
    period: '9.3s',
    delay: '-5s',
    e: 0.74,
  },
  {
    name: 'earth',
    diameterPx: 250,
    sizePx: 6,
    period: '12s',
    delay: '-3s',
    e: 0.76,
  },
  {
    name: 'mars',
    diameterPx: 310,
    sizePx: 5,
    period: '22.6s',
    delay: '-9s',
    e: 0.78,
  },
  {
    name: 'jupiter',
    diameterPx: 420,
    sizePx: 10,
    period: '142s',
    delay: '-15s',
    e: 0.8,
  },
  {
    name: 'saturn',
    diameterPx: 520,
    sizePx: 8,
    period: '353s',
    delay: '-8s',
    e: 0.82,
  },
  {
    name: 'uranus',
    diameterPx: 600,
    sizePx: 7,
    period: '1005s',
    delay: '-20s',
    e: 0.84,
  },
  {
    name: 'neptune',
    diameterPx: 680,
    sizePx: 7,
    period: '1978s',
    delay: '-40s',
    e: 0.86,
  },
  {
    name: 'pluto',
    diameterPx: 740,
    sizePx: 4,
    period: '2950s',
    delay: '-60s',
    e: 0.88,
  },
];

const DEFAULT_ECCENTRICITY = 0.72;

export default function SolarSystemOverlay() {
  const { isDarkMode, mounted } = useStarfield();
  const isVisible = mounted && !isDarkMode;

  return (
    <div
      className={`solar-overlay pointer-events-none fixed inset-0 z-[-1] transition-opacity duration-1000 ease-in-out motion-reduce:transition-none ${isVisible ? 'opacity-100' : 'opacity-0'}`}
      data-paused={!isVisible}
      style={{ '--solar-scale': '1.25' } as React.CSSProperties}
      aria-hidden="true"
    >
      <span className="solar-sun" />
      {PLANETS.map((p) => (
        <div
          key={p.name}
          className="solar-orbit"
          style={
            {
              '--d': `${p.diameterPx}px`,
              '--e': p.e ?? DEFAULT_ECCENTRICITY,
              '--stroke': '1px',
            } as React.CSSProperties
          }
        >
          <span className="solar-scale">
            <span className="solar-orbit-ring" />
            <span
              className="solar-orbit-rot"
              style={
                {
                  animationDelay: p.delay,
                  '--period': p.period,
                } as React.CSSProperties
              }
            >
              <span
                className="solar-planet"
                style={
                  {
                    '--p-size': `${p.sizePx}px`,
                  } as React.CSSProperties
                }
              />
            </span>
          </span>
        </div>
      ))}
    </div>
  );
}
