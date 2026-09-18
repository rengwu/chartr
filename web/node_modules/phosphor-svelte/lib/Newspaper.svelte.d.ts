import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `NewspaperIcon` instead.
 *
 * @example
 * ```svelte
 * <Newspaper color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Newspaper: Component<IconComponentProps, {}, "">;
type Newspaper = ReturnType<typeof Newspaper>;
export default Newspaper;
