type IconProps = { size?: number };

/** Custom RWA marks — not in @web3icons/react. */
export function TbillLogo({ size = 32 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" fill="none" aria-hidden>
      <circle cx="16" cy="16" r="16" fill="#0E7490" />
      <circle cx="16" cy="16" r="12.5" fill="#164E63" />
      <path
        fill="#A5F3FC"
        d="M10 11.2h12v1.6h-5.2V21h-1.6v-8.2H10v-1.6Z"
      />
      <rect x="8.5" y="22.2" width="15" height="1.4" rx="0.7" fill="#A5F3FC" />
      <rect
        x="10.5"
        y="24.2"
        width="11"
        height="1.1"
        rx="0.55"
        fill="#ffffff"
        opacity="0.7"
      />
    </svg>
  );
}

export function TreitLogo({ size = 32 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" fill="none" aria-hidden>
      <circle cx="16" cy="16" r="16" fill="#5B21B6" />
      <path
        fill="#E9D5FF"
        d="M7.5 22.8V14.2l8.5-6.4 8.5 6.4v8.6h-3.2v-5.4h-4.2v5.4h-2.2v-5.4h-4.2v5.4H7.5Z"
      />
      <rect x="14.6" y="17.4" width="2.8" height="5.4" fill="#ffffff" />
    </svg>
  );
}

export function TinvLogo({ size = 32 }: IconProps) {
  return (
    <svg width={size} height={size} viewBox="0 0 32 32" fill="none" aria-hidden>
      <circle cx="16" cy="16" r="16" fill="#B45309" />
      <path fill="#FEF3C7" d="M11 8.4h8.2l3.8 3.8V23.6H11V8.4Z" />
      <path fill="#F59E0B" d="M19.2 8.4v3.8h3.8" />
      <rect x="13" y="14.2" width="6.8" height="1.3" rx="0.5" fill="#B45309" />
      <rect x="13" y="17" width="6.8" height="1.3" rx="0.5" fill="#B45309" />
      <rect x="13" y="19.8" width="4.4" height="1.3" rx="0.5" fill="#D97706" />
    </svg>
  );
}
