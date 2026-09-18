import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CigaretteIcon` instead.
 *
 * @example
 * ```svelte
 * <Cigarette color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Cigarette: Component<IconComponentProps, {}, "">;
type Cigarette = ReturnType<typeof Cigarette>;
export default Cigarette;
