import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `CloudCheckIcon` instead.
 *
 * @example
 * ```svelte
 * <CloudCheck color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const CloudCheck: Component<IconComponentProps, {}, "">;
type CloudCheck = ReturnType<typeof CloudCheck>;
export default CloudCheck;
