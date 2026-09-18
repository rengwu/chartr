import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `SealCheckIcon` instead.
 *
 * @example
 * ```svelte
 * <SealCheck color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const SealCheck: Component<IconComponentProps, {}, "">;
type SealCheck = ReturnType<typeof SealCheck>;
export default SealCheck;
