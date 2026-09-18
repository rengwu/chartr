import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `BabyIcon` instead.
 *
 * @example
 * ```svelte
 * <Baby color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Baby: Component<IconComponentProps, {}, "">;
type Baby = ReturnType<typeof Baby>;
export default Baby;
