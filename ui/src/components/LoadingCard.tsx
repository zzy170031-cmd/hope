interface LoadingCardProps {
  lines?: number;
}

export function LoadingCard({ lines = 3 }: LoadingCardProps) {
  return (
    <div className="loading-card" aria-busy="true" aria-live="polite">
      {Array.from({ length: lines }).map((_, index) => (
        <div key={index} className="loading-card__line" />
      ))}
    </div>
  );
}
