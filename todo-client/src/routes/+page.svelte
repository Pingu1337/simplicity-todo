<script lang="ts">
  import { browser } from "$app/environment";
  import { Button } from "flowbite-svelte";
  import { Label, Input } from "flowbite-svelte";
  import collapse from "svelte-collapse";

  let open = false;

  let storedUser: string | null;

  let inputUser: string | null = null;

  if (browser) {
    storedUser = window.localStorage.getItem("user");
    if (storedUser) window.location.replace(`/kanban?user=${storedUser}`);
  }

  const setUser = (user: string) => window.localStorage.setItem("user", user);

  const login = async (e: any) => {
    e.preventDefault();
    if (inputUser) {
      setUser(inputUser);
      window.location.replace(`/kanban?user=${inputUser}`);
      return;
    }
    const response = await fetch("/api/login", { method: "POST" });
    const user = await response.json();

    setUser(user.id);
    window.location.reload();
  };

  $: user = storedUser;
</script>

{#if !user}
  <form on:submit={login}>
    <div class="flex justify-center items-center flex-col mt-12 mx-auto">
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <span
        class="btn !bg-transparent bg-none border-none outline-none cursor-pointer select-none w-1/4"
        on:click={() => (open = !open)}>Login with user code</span
      >

      <div use:collapse={{ open }} class="mb-6 lg:w-1/4 w-4/5">
        <Input
          id="user-code-input"
          bind:value={inputUser}
          size="lg"
          placeholder="User code"
        />
      </div>
      <Button class="btn btn-primary w-1/4" on:click={login}>Login</Button>
    </div>
  </form>
{/if}
