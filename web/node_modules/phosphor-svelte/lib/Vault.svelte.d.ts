import type { Component } from "svelte";
import type { IconComponentProps } from "./shared.d.ts";

/**
 * @deprecated Use `VaultIcon` instead.
 *
 * @example
 * ```svelte
 * <Vault color="white" weight="fill" size="20px" mirrored={false} />
 * ```
 *
 * @prop {string} color
 * @prop {number | string} size
 * @prop {"bold" | "duotone" | "fill" | "light" | "thin" | "regular"} weight
 * @prop {boolean} mirrored
 */
declare const Vault: Component<IconComponentProps, {}, "">;
type Vault = ReturnType<typeof Vault>;
export default Vault;
