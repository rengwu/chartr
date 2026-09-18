import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `StoolIcon` instead.
 *
 * @example
 * ```svelte
 * <Stool color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Stool: Component<IconComponentProps, {}, "">;
type Stool = ReturnType<typeof Stool>;
export default Stool;
