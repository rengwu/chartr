import type { Snippet } from "svelte";
import type { SVGAttributes } from "svelte/elements";

export type IconWeight =
  | "bold"
  | "duotone"
  | "fill"
  | "light"
  | "thin"
  | "regular";

export interface IconBaseProps {
  /**
   * @type {string}
   * @default "currentColor"
   */
  color?: string;
  /**
   * @type {number|string}
   * @default "1em"
   */
  size?: number | string;
  /**
   * @type {IconWeight}
   * @default "regular"
   */
  weight?: IconWeight;
  /**
   * @type {boolean}
   * @default false
   */
  mirrored?: boolean;
}

export interface IconComponentProps
  extends Omit<SVGAttributes<SVGSVGElement>, keyof IconBaseProps>,
    IconBaseProps {}

export interface IconContextProps {
  values: IconComponentProps;
  children?: Snippet;
}
