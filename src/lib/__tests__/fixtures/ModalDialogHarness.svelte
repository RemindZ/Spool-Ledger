<script lang="ts">
  import ModalDialog from "../../components/ModalDialog.svelte";

  let open = $state(false);
  let confirmed = $state(false);
</script>

<button type="button" onclick={() => (open = true)}>Open confirmation</button>
<ModalDialog
  {open}
  title="Commit plan plan-1?"
  confirmLabel="Commit migration"
  onClose={() => (open = false)}
  onConfirm={() => {
    confirmed = true;
    open = false;
  }}
>
  <p>Cloud persistence and AMS visibility are not guaranteed.</p>
</ModalDialog>
<output aria-label="Confirmation result">{confirmed ? "confirmed" : "pending"}</output>
