import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BroomIcon` instead.
 *
 * @example
 * ```svelte
 * <Broom color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Broom: Component<IconComponentProps, {}, "">;
type Broom = ReturnType<typeof Broom>;
export default Broom;
