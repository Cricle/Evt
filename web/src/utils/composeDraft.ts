export interface ComposeDraftAttachmentSelection {
  files: File[];
  mode: 'post' | 'event';
  source: 'quick-media';
}

let pendingAttachmentSelection: ComposeDraftAttachmentSelection | null = null;

export const setPendingAttachmentSelection = (selection: ComposeDraftAttachmentSelection) => {
  pendingAttachmentSelection = selection;
};

export const consumePendingAttachmentSelection = () => {
  const current = pendingAttachmentSelection;
  pendingAttachmentSelection = null;
  return current;
};
