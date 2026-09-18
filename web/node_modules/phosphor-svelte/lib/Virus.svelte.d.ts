import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VirusIcon` instead.
 *
 * @example
 * ```svelte
 * <Virus color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Virus: Component<IconComponentProps, {}, "">;
type Virus = ReturnType<typeof Virus>;
export default Virus;
