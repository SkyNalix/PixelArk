import { useState, useRef, useEffect, ReactNode, useMemo } from 'react';
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip';

interface OverflowTooltipProps {
  tooltip?: ReactNode;
  children: ReactNode;
}

export const OverflowTooltip = ({ children, tooltip = children }: OverflowTooltipProps) => {
  const labelRef = useRef<HTMLDivElement>(null);
  const [isOverflowing, setIsOverflowing] = useState(false);

  useEffect(() => {
    const checkOverflow = () => {
      if (labelRef.current) {
        setIsOverflowing(labelRef.current.scrollWidth > labelRef.current.clientWidth);
      }
    };
    checkOverflow();
    window.addEventListener('resize', checkOverflow);
    return () => window.removeEventListener('resize', checkOverflow);
  }, [children]);

  const element = useMemo(
    () => (
      <div
        ref={labelRef}
        style={{
          textOverflow: 'ellipsis',
          overflow: 'hidden',
          whiteSpace: 'nowrap',
        }}
      >
        {children}
      </div>
    ),
    [children],
  );

  return isOverflowing ? (
    <Tooltip>
      <TooltipTrigger asChild>{element}</TooltipTrigger>
      <TooltipContent>{tooltip}</TooltipContent>
    </Tooltip>
  ) : (
    element
  );
};
