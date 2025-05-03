# Config Menus
The numbers in the brackets represent to which menu should the user continue to.
* When going back to previous menu, put the cursor on the same item that was previously chosen.

## (`FuzzySelect`[1]) Choose a setting to edit
* Change orchestration: Docker-Compose [2]
* Container/pods alias [4]
* 💾 Save and quit
* 🚫 Quit without saving

## (`Select`[2]) Choose orchestration
* 🐋 Docker-Compose ✅ [1]
* ☸️ Kubernetes [3]
* ↩ Return to previous menu [1]

## (`Input`[3]) Choose name space (leave empty to *not* select Kubernetes)
Shows previously saved name space.
* Empty [2]
* None-Empty after Trim [1]

## (`FuzzySelect`[4]) Choose container name
Container names are sorted by insertion order.
* ➕ Add [5]
* ❌ Remove [13]
* `<container_name_1>` [7]
* `<container_name_2>` [7]
* `<...>` [7]
* ↩ Return to previous menu [1]

## (`Input`[5]) Choose new container name (leave empty to not add)
* Empty [4]
* None-Empty after Trim [6]

## (`Input`[6]) Choose new alias for `<CONTAINER_NAME>` (leave empty to not add)
* Empty [7]
* None-Empty after Trim [7]

## (`FuzzySelect`[7]) Editing `<CONTAINER_NAME>`
Aliases are sorted by insertion order.
* ➕ Add alias [6]
* ❌ Remove alias [8]
* ✏️ Rename `<CONTAINER_NAME>` [11]
* ❌ Remove `<CONTAINER_NAME>` [12]
* `<alias_1>` [9]
* `<alias_1>` [9]
* `<...>` [9]
* ↩ Return to previous menu [4]

## (`MultiSelect`[8]) Remove aliases for `<CONTAINER_NAME>` (`q` to return to previous menu without selecting)
Aliases are sorted by insertion order.
* [ ] `<alias_1>`
* [ ] `<alias_2>`
* [ ] `<...>`
* `Enter` [7]
* `q` [7]

## (`Select`[9]) Editing `<ALIAS>` for `<CONTAINER_NAME>`
* ✏️ Rename alias [10]
* ❌ Remove alias [7]
* ➕ Add TTL [15]
  * If TTL already exist will show: ✏️ Edit TTL
* ❌ Remove TTL [9]
  * Won't show this if there is no TTL.
* ↩ Return to previous menu [7]

## (`Input`[10]) Choose new alias for `<ALIAS>` of `<CONTAINER_NAME>` (leave empty to remove it)
* Empty [7]
* None-Empty after Trim [7]

## (`Input`[11]) Choose new container name for `<CONTAINER_NAME>` (leave empty to not rename)
* Empty [7]
* None-Empty after Trim [7]

## (`Confirm`[12]) Are you sure you want to remove `<CONTAINER_NAME>`?
* Yes [4]
* No [7]

## (`MultiSelect`[13]) Remove multiple container names (`q` to return to previous menu without selecting)
Container names are sorted by insertion order.
* [ ] `<container_name_1>`
* [ ] `<container_name_2>`
* [ ] `<...>`
* `Enter` [14]
* `q` [4]

## (`Confirm`[14]) Are you sure you want to remove those containers?
* Yes [4]
* No [13] with the same selected container names.

## (`Input`[15]) Editing TTL for `<ALIAS>` of `<CONTAINER_NAME>` (leave empty to not change the TTL)
Starts filed with the current TTL or 24 hours from now.
* Empty [9]
* None-Empty after Trim [9]
