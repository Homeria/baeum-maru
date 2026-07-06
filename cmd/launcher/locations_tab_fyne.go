//go:build fyne

package main

import (
	"context"
	"fmt"
	"strings"

	"fyne.io/fyne/v2"
	"fyne.io/fyne/v2/container"
	"fyne.io/fyne/v2/theme"
	"fyne.io/fyne/v2/widget"

	"github.com/Homeria/baeum-maru/internal/domain"
	"github.com/Homeria/baeum-maru/internal/service"
)

func buildLocationsTab(state *launcherState) fyne.CanvasObject {
	buildings := []domain.Building{}
	buildingFloors := []domain.BuildingFloor{}
	roleFilter := widget.NewSelect([]string{"전체", "활성", "비활성"}, nil)
	roleFilter.SetSelected("전체")
	roleNameEntry := widget.NewEntry()
	roleNameEntry.SetPlaceHolder("예: 강의, 사무, 접수, 행사")
	roleEditEntry := widget.NewEntry()
	roleEditEntry.SetPlaceHolder("선택한 역할 이름")
	roleStatusSelect := widget.NewSelect(nil, nil)
	buildingNameEntry := widget.NewEntry()
	buildingNameEntry.SetPlaceHolder("예: 본관, 별관")
	buildingEditEntry := widget.NewEntry()
	buildingEditEntry.SetPlaceHolder("선택한 건물 이름")
	buildingStatusSelect := widget.NewSelect(nil, nil)
	var floorBuildingSelect *widget.Select
	floorNameEntry := widget.NewEntry()
	floorNameEntry.SetPlaceHolder("예: B1, 1층, 2층, 옥상")
	floorStatusSelect := widget.NewSelect(nil, nil)
	locationStatusHint := widget.NewLabel("선택한 공간이 없습니다.")
	var locationStatusButton *widget.Button
	var refreshLocationStatusButton func()
	var refreshSelectedLocationViews func(domain.Location)
	var buildingStatusButton *widget.Button
	var refreshBuildingStatusButton func()
	var roleStatusButton *widget.Button
	var refreshRoleStatusButton func()
	var floorStatusButton *widget.Button
	var refreshFloorStatusButton func()
	var locationList *widget.List
	var buildingList *widget.List
	var roleList *widget.List
	var floorList *widget.List
	buildingSelects := []*widget.Select{}
	type floorSelector struct {
		building *widget.Select
		floor    *widget.Select
	}
	floorSelectors := []floorSelector{}
	var refreshFloorSelectors func()

	buildingNameFromOption := func(option string) string {
		if option == "" || option == "건물 없음" {
			return ""
		}
		for _, building := range buildings {
			if buildingOptionLabel(building) == option {
				return building.Name
			}
		}
		return strings.TrimSpace(option)
	}
	buildingOptions := func() []string {
		options := make([]string, 0, len(buildings))
		for _, building := range buildings {
			options = append(options, buildingOptionLabel(building))
		}
		return options
	}
	buildingOptionForName := func(name string) string {
		name = strings.TrimSpace(name)
		if name == "" {
			return "건물 없음"
		}
		for _, building := range buildings {
			if building.Name == name {
				return buildingOptionLabel(building)
			}
		}
		return "건물 없음"
	}
	newBuildingSelect := func() *widget.Select {
		selectWidget := widget.NewSelect([]string{"건물 없음"}, nil)
		selectWidget.SetSelected("건물 없음")
		buildingSelects = append(buildingSelects, selectWidget)
		return selectWidget
	}
	buildingIDFromName := func(name string) int64 {
		name = strings.TrimSpace(name)
		for _, building := range buildings {
			if building.Name == name {
				return building.ID
			}
		}
		return 0
	}
	floorOptionLabel := func(floor domain.BuildingFloor) string {
		if floor.IsActive {
			return floor.Label
		}
		return floor.Label + " (비활성)"
	}
	floorOptionsForBuilding := func(buildingID int64) []string {
		options := []string{"층 없음"}
		if buildingID <= 0 {
			return options
		}
		for _, floor := range buildingFloors {
			if floor.BuildingID == buildingID {
				options = append(options, floorOptionLabel(floor))
			}
		}
		return options
	}
	floorLabelFromOption := func(buildingID int64, option string) string {
		if option == "" || option == "층 없음" {
			return ""
		}
		for _, floor := range buildingFloors {
			if floor.BuildingID == buildingID && floorOptionLabel(floor) == option {
				return floor.Label
			}
		}
		return strings.TrimSpace(option)
	}
	floorOptionForLabel := func(buildingID int64, label string) string {
		label = strings.TrimSpace(label)
		if label == "" {
			return "층 없음"
		}
		for _, floor := range buildingFloors {
			if floor.BuildingID == buildingID && floor.Label == label {
				return floorOptionLabel(floor)
			}
		}
		return label
	}
	findBuildingFloor := func(buildingID int64, option string) (domain.BuildingFloor, bool) {
		label := floorLabelFromOption(buildingID, option)
		if label == "" {
			return domain.BuildingFloor{}, false
		}
		for _, floor := range buildingFloors {
			if floor.BuildingID == buildingID && floor.Label == label {
				return floor, true
			}
		}
		return domain.BuildingFloor{}, false
	}
	newFloorSelect := func(buildingSelect *widget.Select) *widget.Select {
		selectWidget := widget.NewSelect([]string{"층 없음"}, nil)
		selectWidget.SetSelected("층 없음")
		floorSelectors = append(floorSelectors, floorSelector{building: buildingSelect, floor: selectWidget})
		buildingSelect.OnChanged = func(string) {
			refreshFloorSelectors()
		}
		return selectWidget
	}

	roleNameFromOption := func(option string) string {
		for _, role := range state.locationRoles {
			if locationRoleOptionLabel(role.Name) == option || locationRoleManageOptionLabel(role) == option {
				return role.Name
			}
		}
		return strings.TrimSpace(option)
	}
	newRolePicker := func() (fyne.CanvasObject, func() []string, func([]string), func()) {
		selectedRoles := []string{}
		selectedRoleSummary := widget.NewLabel("선택된 역할 없음")
		selectedRoleSummary.Wrapping = fyne.TextWrapWord
		roleButtonBox := container.NewGridWrap(fyne.NewSize(118, 36))
		roleButtonScroll := container.NewVScroll(roleButtonBox)
		roleButtonScroll.SetMinSize(fyne.NewSize(360, 148))

		var refreshRoleButtons func()
		selectedRoleNames := func() []string {
			roles := make([]string, len(selectedRoles))
			copy(roles, selectedRoles)
			return roles
		}
		setSelectedRoleNames := func(roles []string) {
			selectedRoles = uniqueLauncherRoles(roles)
			refreshRoleButtons()
		}
		toggleSelectedRole := func(role string) {
			role = normalizeLauncherLocationRole(role)
			if role == "" {
				return
			}
			if hasLauncherLocationRole(selectedRoles, role) {
				next := make([]string, 0, len(selectedRoles))
				for _, selected := range selectedRoles {
					if normalizeLauncherLocationRole(selected) != role {
						next = append(next, selected)
					}
				}
				selectedRoles = next
			} else {
				selectedRoles = append(selectedRoles, role)
			}
			refreshRoleButtons()
		}
		refreshRoleButtons = func() {
			if len(selectedRoles) == 0 {
				selectedRoleSummary.SetText("선택된 역할 없음")
			} else {
				selectedRoleSummary.SetText("선택됨: " + locationRoleLabel(selectedRoles))
			}

			roleButtonBox.Objects = nil
			for _, role := range state.locationRoles {
				if !role.IsActive {
					continue
				}
				roleName := role.Name
				button := widget.NewButton(locationRoleOptionLabel(roleName), func() {
					toggleSelectedRole(roleName)
				})
				if hasLauncherLocationRole(selectedRoles, roleName) {
					button.Importance = widget.HighImportance
				} else {
					button.Importance = widget.LowImportance
				}
				roleButtonBox.Add(button)
			}
			roleButtonBox.Refresh()
		}
		refreshRoleButtons()
		return container.NewVBox(selectedRoleSummary, roleButtonScroll), selectedRoleNames, setSelectedRoleNames, refreshRoleButtons
	}

	addRolePicker, addSelectedRoleNames, setAddSelectedRoleNames, refreshAddRoleButtons := newRolePicker()
	editRolePicker, editSelectedRoleNames, setEditSelectedRoleNames, refreshEditRoleButtons := newRolePicker()

	roleOptions := func() []string {
		options := make([]string, 0, len(state.locationRoles))
		for _, role := range state.locationRoles {
			if role.IsActive {
				options = append(options, locationRoleOptionLabel(role.Name))
			}
		}
		return options
	}
	refreshRoleControls := func() {
		options := roleOptions()
		refreshAddRoleButtons()
		refreshEditRoleButtons()

		filterOptions := append([]string{"전체", "활성", "비활성"}, options...)
		roleFilter.Options = filterOptions
		if roleFilter.Selected == "" || !containsString(filterOptions, roleFilter.Selected) {
			roleFilter.Selected = "전체"
			roleFilter.Refresh()
		} else {
			roleFilter.Refresh()
		}

		managementOptions := make([]string, 0, len(state.locationRoles))
		for _, role := range state.locationRoles {
			managementOptions = append(managementOptions, locationRoleManageOptionLabel(role))
		}
		roleStatusSelect.Options = managementOptions
		if roleStatusSelect.Selected != "" && !containsString(managementOptions, roleStatusSelect.Selected) {
			roleStatusSelect.Selected = ""
			roleEditEntry.SetText("")
			roleStatusSelect.Refresh()
		} else {
			roleStatusSelect.Refresh()
		}
		if roleList != nil {
			roleList.Refresh()
		}
		if refreshRoleStatusButton != nil {
			refreshRoleStatusButton()
		}
	}
	refreshRoles := func() {
		roles, err := state.runtime.Locations.ListRoles(context.Background(), service.LocationRoleListInput{
			IncludeInactive: true,
			Limit:           500,
		})
		if err != nil {
			state.setStatus("역할 목록 오류", "공간 역할 목록 조회 실패: "+err.Error())
			return
		}
		state.locationRoles = roles
		refreshRoleControls()
	}
	refreshBuildingControls := func() {
		selectOptions := append([]string{"건물 없음"}, buildingOptions()...)
		for _, buildingSelect := range buildingSelects {
			current := buildingSelect.Selected
			buildingSelect.Options = selectOptions
			if current == "" || !containsString(selectOptions, current) {
				buildingSelect.Selected = "건물 없음"
			}
			buildingSelect.Refresh()
		}

		deleteOptions := buildingOptions()
		buildingStatusSelect.Options = deleteOptions
		if buildingStatusSelect.Selected != "" && !containsString(deleteOptions, buildingStatusSelect.Selected) {
			buildingStatusSelect.Selected = ""
		}
		buildingStatusSelect.Refresh()

		if buildingList != nil {
			buildingList.Refresh()
		}
		if refreshBuildingStatusButton != nil {
			refreshBuildingStatusButton()
		}
		if refreshFloorSelectors != nil {
			refreshFloorSelectors()
		}
	}
	refreshBuildings := func() {
		items, err := state.runtime.Locations.ListBuildings(context.Background(), service.BuildingListInput{
			IncludeInactive: true,
			Limit:           500,
		})
		if err != nil {
			state.setStatus("건물 목록 오류", "건물 목록 조회 실패: "+err.Error())
			return
		}
		buildings = items
		refreshBuildingControls()
	}
	refreshFloorSelectors = func() {
		for _, selector := range floorSelectors {
			buildingID := buildingIDFromName(buildingNameFromOption(selector.building.Selected))
			current := selector.floor.Selected
			options := floorOptionsForBuilding(buildingID)
			selector.floor.Options = options
			if current == "" || !containsString(options, current) {
				selector.floor.Selected = "층 없음"
			}
			selector.floor.Refresh()
		}
	}
	refreshFloorControls := func() {
		buildingID := buildingIDFromName(buildingNameFromOption(floorBuildingSelect.Selected))
		options := floorOptionsForBuilding(buildingID)
		floorStatusSelect.Options = options[1:]
		if floorStatusSelect.Selected != "" && !containsString(floorStatusSelect.Options, floorStatusSelect.Selected) {
			floorStatusSelect.Selected = ""
		}
		floorStatusSelect.Refresh()
		refreshFloorSelectors()
		if floorList != nil {
			floorList.Refresh()
		}
		if refreshFloorStatusButton != nil {
			refreshFloorStatusButton()
		}
	}
	refreshBuildingFloors := func() {
		items, err := state.runtime.Locations.ListBuildingFloors(context.Background(), service.BuildingFloorListInput{
			IncludeInactive: true,
			Limit:           1000,
		})
		if err != nil {
			state.setStatus("층 목록 오류", "건물 층 목록 조회 실패: "+err.Error())
			return
		}
		buildingFloors = items
		refreshFloorControls()
	}

	addNameEntry := widget.NewEntry()
	addNameEntry.SetPlaceHolder("예: 문화교육실, 2층 다용도실")
	addBuildingSelect := newBuildingSelect()
	addFloorSelect := newFloorSelect(addBuildingSelect)
	addNoteEntry := widget.NewMultiLineEntry()
	addNoteEntry.SetPlaceHolder("시설 특징, 사용 제한, 내부 운영 메모")
	addNoteEntry.SetMinRowsVisible(3)
	addActiveCheck := widget.NewCheck("처음부터 활성 상태로 등록", nil)
	addActiveCheck.SetChecked(true)

	editNameEntry := widget.NewEntry()
	editNameEntry.SetPlaceHolder("예: 문화교육실, 2층 다용도실")
	editBuildingSelect := newBuildingSelect()
	editFloorSelect := newFloorSelect(editBuildingSelect)
	editNoteEntry := widget.NewMultiLineEntry()
	editNoteEntry.SetPlaceHolder("시설 특징, 사용 제한, 내부 운영 메모")
	editNoteEntry.SetMinRowsVisible(3)
	editActiveCheck := widget.NewCheck("저장 시 활성 상태로 둠", nil)
	editActiveCheck.SetChecked(true)
	floorBuildingSelect = newBuildingSelect()
	floorBuildingSelect.OnChanged = func(string) {
		floorStatusSelect.Selected = ""
		floorStatusSelect.Refresh()
		refreshFloorControls()
	}

	resetAddForm := func() {
		addNameEntry.SetText("")
		addBuildingSelect.SetSelected("건물 없음")
		addFloorSelect.SetSelected("층 없음")
		addNoteEntry.SetText("")
		setAddSelectedRoleNames([]string{"common"})
		addActiveCheck.SetChecked(true)
	}
	resetEditForm := func() {
		editNameEntry.SetText("")
		editBuildingSelect.SetSelected("건물 없음")
		editFloorSelect.SetSelected("층 없음")
		editNoteEntry.SetText("")
		setEditSelectedRoleNames([]string{"common"})
		editActiveCheck.SetChecked(true)
		state.selectedLocationID = 0
		if refreshLocationStatusButton != nil {
			refreshLocationStatusButton()
		}
		for _, list := range state.locationLists {
			list.UnselectAll()
		}
	}

	inputFromFields := func(name string, buildingOption string, floor string, note string, active bool, roles []string) service.LocationInput {
		return service.LocationInput{
			Name:        name,
			Building:    buildingNameFromOption(buildingOption),
			Floor:       floor,
			Type:        preferredLauncherLocationRole(roles),
			Roles:       roles,
			IsClassroom: hasLauncherLocationRole(roles, domain.LocationTypeClassroom),
			IsActive:    active,
			Note:        note,
		}
	}
	inputFromAddForm := func() service.LocationInput {
		buildingID := buildingIDFromName(buildingNameFromOption(addBuildingSelect.Selected))
		return inputFromFields(addNameEntry.Text, addBuildingSelect.Selected, floorLabelFromOption(buildingID, addFloorSelect.Selected), addNoteEntry.Text, addActiveCheck.Checked, addSelectedRoleNames())
	}
	inputFromEditForm := func() service.LocationInput {
		buildingID := buildingIDFromName(buildingNameFromOption(editBuildingSelect.Selected))
		return inputFromFields(editNameEntry.Text, editBuildingSelect.Selected, floorLabelFromOption(buildingID, editFloorSelect.Selected), editNoteEntry.Text, editActiveCheck.Checked, editSelectedRoleNames())
	}

	fillEditForm := func(location domain.Location) {
		editNameEntry.SetText(location.Name)
		editBuildingSelect.SetSelected(buildingOptionForName(location.Building))
		buildingID := buildingIDFromName(location.Building)
		editFloorSelect.SetSelected(floorOptionForLabel(buildingID, location.Floor))
		editNoteEntry.SetText(location.Note)
		setEditSelectedRoleNames(locationRolesForUI(location))
		editActiveCheck.SetChecked(location.IsActive)
		if refreshLocationStatusButton != nil {
			refreshLocationStatusButton()
		}
	}

	createButton := widget.NewButtonWithIcon("등록", theme.ContentAddIcon(), func() {
		created, err := state.runtime.Locations.Create(context.Background(), inputFromAddForm())
		if err != nil {
			state.setStatus("공간 등록 실패", "공간 등록 실패: "+err.Error())
			return
		}
		state.setStatus("공간 등록 완료", created.Name+" 공간을 등록했습니다.")
		resetAddForm()
		state.refreshLocations()
	})
	createButton.Importance = widget.HighImportance
	updateButton := widget.NewButtonWithIcon("수정하기", theme.DocumentSaveIcon(), func() {
		if state.selectedLocationID <= 0 {
			state.setStatus("공간 선택 필요", "수정할 공간을 선택하세요.")
			return
		}
		updated, err := state.runtime.Locations.Update(context.Background(), state.selectedLocationID, inputFromEditForm())
		if err != nil {
			state.setStatus("공간 수정 실패", "공간 수정 실패: "+err.Error())
			return
		}
		state.setStatus("공간 수정 완료", updated.Name+" 공간을 저장했습니다.")
		state.refreshLocations()
		state.selectedLocationID = updated.ID
		fillEditForm(updated)
		if refreshSelectedLocationViews != nil {
			refreshSelectedLocationViews(updated)
		}
		refreshLocationStatusButton()
	})
	updateButton.Importance = widget.HighImportance
	refreshLocationStatusButton = func() {
		if locationStatusButton == nil {
			return
		}
		location, ok := state.selectedLocation()
		if !ok {
			locationStatusHint.SetText("선택한 공간이 없습니다.")
			locationStatusButton.SetText("공간 선택 필요")
			locationStatusButton.SetIcon(theme.HelpIcon())
			locationStatusButton.Importance = widget.LowImportance
			locationStatusButton.Refresh()
			return
		}
		if location.IsActive {
			locationStatusHint.SetText(location.Name + " 공간은 현재 활성화 상태입니다.")
			locationStatusButton.SetText("비활성화로 변경")
			locationStatusButton.SetIcon(theme.DeleteIcon())
			locationStatusButton.Importance = widget.LowImportance
		} else {
			locationStatusHint.SetText(location.Name + " 공간은 현재 비활성화 상태입니다.")
			locationStatusButton.SetText("활성화로 변경")
			locationStatusButton.SetIcon(theme.ConfirmIcon())
			locationStatusButton.Importance = widget.HighImportance
		}
		locationStatusButton.Refresh()
	}
	toggleLocationStatus := func() {
		location, ok := state.selectedLocation()
		if !ok {
			state.setStatus("공간 선택 필요", "상태를 변경할 공간을 선택하세요.")
			return
		}
		nextActive := !location.IsActive
		updated, err := state.runtime.Locations.Update(context.Background(), location.ID, service.LocationInput{
			Name:        location.Name,
			Building:    location.Building,
			Floor:       location.Floor,
			Type:        location.Type,
			Roles:       locationRolesForUI(location),
			IsClassroom: location.IsClassroom,
			IsActive:    nextActive,
			Note:        location.Note,
		})
		if err != nil {
			state.setStatus("공간 상태 변경 실패", "공간 상태 변경 실패: "+err.Error())
			return
		}
		status := "비활성화"
		if updated.IsActive {
			status = "활성화"
		}
		state.refreshLocations()
		state.selectedLocationID = updated.ID
		fillEditForm(updated)
		if refreshSelectedLocationViews != nil {
			refreshSelectedLocationViews(updated)
		}
		if locationList != nil {
			locationList.Refresh()
		}
		refreshLocationStatusButton()
		state.setStatus("공간 상태 변경 완료", updated.Name+" 공간을 "+status+"했습니다.")
	}
	locationStatusButton = widget.NewButtonWithIcon("공간 선택 필요", theme.HelpIcon(), toggleLocationStatus)
	refreshLocationStatusButton()
	clearAddButton := widget.NewButtonWithIcon("새 입력", theme.ViewRefreshIcon(), resetAddForm)
	clearEditButton := widget.NewButtonWithIcon("선택 해제", theme.ViewRefreshIcon(), resetEditForm)

	createRoleButton := widget.NewButtonWithIcon("역할 추가", theme.ContentAddIcon(), func() {
		role, err := state.runtime.Locations.CreateRole(context.Background(), roleNameEntry.Text)
		if err != nil {
			state.setStatus("역할 추가 실패", "공간 역할 추가 실패: "+err.Error())
			return
		}
		roleNameEntry.SetText("")
		refreshRoles()
		roleStatusSelect.SetSelected(locationRoleManageOptionLabel(role))
		roleEditEntry.SetText(role.Name)
		state.setStatus("역할 추가 완료", locationRoleOptionLabel(role.Name)+" 역할을 추가했습니다. 역할 버튼에서 선택할 수 있습니다.")
	})
	updateRoleButton := widget.NewButtonWithIcon("역할 수정", theme.DocumentSaveIcon(), func() {
		role, ok := findLauncherLocationRole(state.locationRoles, roleNameFromOption(roleStatusSelect.Selected))
		if !ok {
			state.setStatus("역할 선택 필요", "수정할 공간 역할을 선택하세요.")
			return
		}
		updated, err := state.runtime.Locations.UpdateRole(context.Background(), role.ID, roleEditEntry.Text)
		if err != nil {
			state.setStatus("역할 수정 실패", "공간 역할 수정 실패: "+err.Error())
			return
		}
		refreshRoles()
		roleStatusSelect.SetSelected(locationRoleManageOptionLabel(updated))
		roleEditEntry.SetText(updated.Name)
		state.refreshLocations()
		state.setStatus("역할 수정 완료", locationRoleOptionLabel(updated.Name)+" 역할을 저장했습니다.")
	})
	updateRoleButton.Importance = widget.HighImportance
	refreshRoleStatusButton = func() {
		if roleStatusButton == nil {
			return
		}
		role, ok := findLauncherLocationRole(state.locationRoles, roleNameFromOption(roleStatusSelect.Selected))
		if !ok {
			roleStatusButton.SetText("역할 선택 필요")
			roleStatusButton.SetIcon(theme.HelpIcon())
			roleStatusButton.Importance = widget.LowImportance
			roleStatusButton.Refresh()
			return
		}
		roleEditEntry.SetText(role.Name)
		if role.IsActive {
			roleStatusButton.SetText("비활성화로 변경")
			roleStatusButton.SetIcon(theme.DeleteIcon())
			roleStatusButton.Importance = widget.LowImportance
		} else {
			roleStatusButton.SetText("활성화로 변경")
			roleStatusButton.SetIcon(theme.ConfirmIcon())
			roleStatusButton.Importance = widget.HighImportance
		}
		roleStatusButton.Refresh()
	}
	toggleRoleStatus := func() {
		role, ok := findLauncherLocationRole(state.locationRoles, roleNameFromOption(roleStatusSelect.Selected))
		if !ok {
			state.setStatus("역할 선택 필요", "상태를 변경할 공간 역할을 선택하세요.")
			return
		}
		nextActive := !role.IsActive
		var err error
		if nextActive {
			err = state.runtime.Locations.ActivateRole(context.Background(), role.ID)
		} else {
			err = state.runtime.Locations.DeactivateRole(context.Background(), role.ID)
		}
		if err != nil {
			state.setStatus("역할 상태 변경 실패", "공간 역할 상태 변경 실패: "+err.Error())
			return
		}
		status := "비활성화"
		if nextActive {
			status = "활성화"
		}
		refreshRoles()
		roleStatusSelect.SetSelected(locationRoleOptionLabel(role.Name))
		state.refreshLocations()
		state.setStatus("역할 상태 변경 완료", locationRoleOptionLabel(role.Name)+" 역할을 "+status+"했습니다.")
	}
	roleStatusButton = widget.NewButtonWithIcon("역할 선택 필요", theme.HelpIcon(), toggleRoleStatus)
	roleStatusSelect.OnChanged = func(string) {
		refreshRoleStatusButton()
	}
	deleteRoleButton := widget.NewButtonWithIcon("역할 삭제", theme.DeleteIcon(), func() {
		role, ok := findLauncherLocationRole(state.locationRoles, roleNameFromOption(roleStatusSelect.Selected))
		if !ok {
			state.setStatus("역할 선택 필요", "삭제할 공간 역할을 선택하세요.")
			return
		}
		if err := state.runtime.Locations.DeleteRole(context.Background(), role.ID); err != nil {
			state.setStatus("역할 삭제 실패", "공간 역할 삭제 실패: "+err.Error())
			return
		}
		setAddSelectedRoleNames(withoutLauncherRole(addSelectedRoleNames(), role.Name))
		setEditSelectedRoleNames(withoutLauncherRole(editSelectedRoleNames(), role.Name))
		if normalizeLauncherLocationRole(state.locationFilter) == normalizeLauncherLocationRole(role.Name) {
			state.setLocationFilter("all")
			roleFilter.Selected = "전체"
			roleFilter.Refresh()
		}
		roleStatusSelect.Selected = ""
		roleEditEntry.SetText("")
		roleStatusSelect.Refresh()
		refreshRoles()
		state.refreshLocations()
		state.setStatus("역할 삭제 완료", locationRoleOptionLabel(role.Name)+" 역할을 삭제했습니다.")
	})

	addForm := widget.NewForm(
		widget.NewFormItem("공간명", addNameEntry),
		widget.NewFormItem("건물", addBuildingSelect),
		widget.NewFormItem("층", addFloorSelect),
		widget.NewFormItem("역할", container.NewVBox(
			addRolePicker,
		)),
		widget.NewFormItem("운영 메모", addNoteEntry),
	)
	addLocationList := buildLocationInventoryList(state)
	state.addLocationList(addLocationList)
	addRefreshButton := widget.NewButtonWithIcon("새로고침", theme.ViewRefreshIcon(), state.refreshLocations)
	addPanel := container.NewHSplit(
		widget.NewCard("공간 추가", "", container.NewVBox(
			addForm,
			addActiveCheck,
			container.NewHBox(createButton, clearAddButton),
		)),
		widget.NewCard("등록된 공간", "", container.NewBorder(
			nil,
			container.NewHBox(addRefreshButton),
			nil,
			nil,
			addLocationList,
		)),
	)
	addPanel.SetOffset(0.42)

	editForm := widget.NewForm(
		widget.NewFormItem("공간명", editNameEntry),
		widget.NewFormItem("건물", editBuildingSelect),
		widget.NewFormItem("층", editFloorSelect),
		widget.NewFormItem("역할", container.NewVBox(
			editRolePicker,
		)),
		widget.NewFormItem("운영 메모", editNoteEntry),
	)
	editFormCard := widget.NewCard("공간 정보 수정", "", container.NewVBox(
		container.NewHBox(updateButton, clearEditButton),
		widget.NewSeparator(),
		widget.NewLabelWithStyle("공간 상태 변경", fyne.TextAlignLeading, fyne.TextStyle{Bold: true}),
		locationStatusHint,
		locationStatusButton,
		widget.NewSeparator(),
		editForm,
		editActiveCheck,
	))

	selectedTitle := widget.NewLabelWithStyle("선택한 공간이 없습니다.", fyne.TextAlignLeading, fyne.TextStyle{Bold: true})
	selectedMeta := widget.NewLabel("-")
	selectedMeta.Wrapping = fyne.TextWrapWord
	selectedNote := widget.NewLabel("-")
	selectedNote.Wrapping = fyne.TextWrapWord
	showDetails := func(location domain.Location) {
		selectedTitle.SetText(location.Name)
		selectedMeta.SetText(fmt.Sprintf("%s / %s / %s", locationRoleLabel(locationRolesForUI(location)), locationStatusLabel(location), locationAddressLabel(location)))
		selectedNote.SetText(emptyDash(location.Note))
	}
	resetDetails := func() {
		selectedTitle.SetText("선택한 공간이 없습니다.")
		selectedMeta.SetText("-")
		selectedNote.SetText("-")
	}
	refreshSelectedLocationViews = showDetails

	locationList = buildLocationList(state)
	locationList.OnSelected = func(id widget.ListItemID) {
		location, ok := state.selectLocation(id)
		if !ok {
			resetDetails()
			refreshLocationStatusButton()
			return
		}
		fillEditForm(location)
		showDetails(location)
		refreshLocationStatusButton()
	}
	state.addLocationList(locationList)

	roleFilter.OnChanged = func(selected string) {
		if selected == "전체" {
			state.setLocationFilter("all")
		} else if selected == "활성" {
			state.setLocationFilter("active")
		} else if selected == "비활성" {
			state.setLocationFilter("inactive")
		} else {
			state.setLocationFilter(roleNameFromOption(selected))
		}
		resetDetails()
		resetEditForm()
	}
	refreshButton := widget.NewButtonWithIcon("새로고침", theme.ViewRefreshIcon(), func() {
		refreshRoles()
		state.refreshLocations()
		resetDetails()
		refreshLocationStatusButton()
	})

	listPane := widget.NewCard("공간 목록", "", container.NewBorder(
		container.NewVBox(roleFilter),
		container.NewHBox(refreshButton),
		nil,
		nil,
		locationList,
	))
	detailPane := widget.NewCard("선택 상세", "", container.NewVBox(
		selectedTitle,
		selectedMeta,
		widget.NewSeparator(),
		locationDetailRow("운영 메모", selectedNote),
	))
	editRightSplit := container.NewVSplit(listPane, detailPane)
	editRightSplit.SetOffset(0.70)
	editPanel := container.NewHSplit(editFormCard, editRightSplit)
	editPanel.SetOffset(0.42)

	roleList = widget.NewList(
		func() int { return len(state.locationRoles) },
		func() fyne.CanvasObject {
			name := widget.NewLabel("")
			name.TextStyle = fyne.TextStyle{Bold: true}
			meta := widget.NewLabel("")
			return container.NewVBox(name, meta)
		},
		func(id widget.ListItemID, item fyne.CanvasObject) {
			if id < 0 || id >= len(state.locationRoles) {
				return
			}
			box := item.(*fyne.Container)
			name := box.Objects[0].(*widget.Label)
			meta := box.Objects[1].(*widget.Label)
			role := state.locationRoles[id]
			name.SetText(locationRoleOptionLabel(role.Name))
			if role.IsActive {
				meta.SetText("사용")
			} else {
				meta.SetText("비활성")
			}
		},
	)
	roleList.OnSelected = func(id widget.ListItemID) {
		if id < 0 || id >= len(state.locationRoles) {
			roleStatusSelect.SetSelected("")
			roleEditEntry.SetText("")
			refreshRoleStatusButton()
			return
		}
		role := state.locationRoles[id]
		roleStatusSelect.SetSelected(locationRoleManageOptionLabel(role))
		roleEditEntry.SetText(role.Name)
		refreshRoleStatusButton()
	}
	refreshRoleButton := widget.NewButtonWithIcon("새로고침", theme.ViewRefreshIcon(), refreshRoles)
	roleControlPane := container.NewVBox(
		widget.NewCard("역할 추가", "", container.NewBorder(nil, nil, nil, createRoleButton, roleNameEntry)),
		widget.NewCard("역할 수정", "", container.NewVBox(
			roleStatusSelect,
			roleEditEntry,
			container.NewHBox(updateRoleButton, deleteRoleButton),
		)),
		widget.NewCard("역할 상태 변경", "", container.NewVBox(
			roleStatusButton,
		)),
		refreshRoleButton,
	)
	rolePanel := container.NewHSplit(
		roleControlPane,
		widget.NewCard("역할 목록", "", roleList),
	)
	rolePanel.SetOffset(0.38)

	createBuildingButton := widget.NewButtonWithIcon("건물 추가", theme.ContentAddIcon(), func() {
		building, err := state.runtime.Locations.CreateBuilding(context.Background(), buildingNameEntry.Text)
		if err != nil {
			state.setStatus("건물 추가 실패", "건물 추가 실패: "+err.Error())
			return
		}
		buildingNameEntry.SetText("")
		refreshBuildings()
		addBuildingSelect.SetSelected(buildingOptionForName(building.Name))
		state.setStatus("건물 추가 완료", building.Name+" 건물을 추가했습니다.")
	})
	updateBuildingButton := widget.NewButtonWithIcon("건물 수정", theme.DocumentSaveIcon(), func() {
		building, ok := findLauncherBuilding(buildings, buildingNameFromOption(buildingStatusSelect.Selected))
		if !ok {
			state.setStatus("건물 선택 필요", "수정할 건물을 선택하세요.")
			return
		}
		updated, err := state.runtime.Locations.UpdateBuilding(context.Background(), building.ID, buildingEditEntry.Text)
		if err != nil {
			state.setStatus("건물 수정 실패", "건물 수정 실패: "+err.Error())
			return
		}
		refreshBuildings()
		buildingStatusSelect.SetSelected(buildingOptionForName(updated.Name))
		buildingEditEntry.SetText(updated.Name)
		state.refreshLocations()
		state.setStatus("건물 수정 완료", updated.Name+" 건물을 저장했습니다.")
	})
	updateBuildingButton.Importance = widget.HighImportance
	deleteBuildingButton := widget.NewButtonWithIcon("건물 삭제", theme.DeleteIcon(), func() {
		building, ok := findLauncherBuilding(buildings, buildingNameFromOption(buildingStatusSelect.Selected))
		if !ok {
			state.setStatus("건물 선택 필요", "삭제할 건물을 선택하세요.")
			return
		}
		if err := state.runtime.Locations.DeleteBuilding(context.Background(), building.ID); err != nil {
			state.setStatus("건물 삭제 실패", "공간이나 층에서 사용 중이면 삭제할 수 없습니다: "+err.Error())
			return
		}
		buildingStatusSelect.Selected = ""
		buildingEditEntry.SetText("")
		buildingStatusSelect.Refresh()
		refreshBuildings()
		refreshBuildingFloors()
		state.refreshLocations()
		state.setStatus("건물 삭제 완료", building.Name+" 건물을 삭제했습니다.")
	})
	createFloorButton := widget.NewButtonWithIcon("층 추가", theme.ContentAddIcon(), func() {
		building, ok := findLauncherBuilding(buildings, buildingNameFromOption(floorBuildingSelect.Selected))
		if !ok {
			state.setStatus("건물 선택 필요", "층을 추가할 건물을 선택하세요.")
			return
		}
		floor, err := state.runtime.Locations.CreateBuildingFloor(context.Background(), building.ID, floorNameEntry.Text)
		if err != nil {
			state.setStatus("층 추가 실패", "건물 층 추가 실패: "+err.Error())
			return
		}
		floorNameEntry.SetText("")
		refreshBuildingFloors()
		floorStatusSelect.SetSelected(floorOptionLabel(floor))
		state.setStatus("층 추가 완료", building.Name+" "+floor.Label+" 층을 추가했습니다.")
	})
	refreshBuildingStatusButton = func() {
		if buildingStatusButton == nil {
			return
		}
		building, ok := findLauncherBuilding(buildings, buildingNameFromOption(buildingStatusSelect.Selected))
		if !ok {
			buildingStatusButton.SetText("건물 선택 필요")
			buildingStatusButton.SetIcon(theme.HelpIcon())
			buildingStatusButton.Importance = widget.LowImportance
			buildingStatusButton.Refresh()
			return
		}
		if building.IsActive {
			buildingStatusButton.SetText("비활성화로 변경")
			buildingStatusButton.SetIcon(theme.DeleteIcon())
			buildingStatusButton.Importance = widget.LowImportance
		} else {
			buildingStatusButton.SetText("활성화로 변경")
			buildingStatusButton.SetIcon(theme.ConfirmIcon())
			buildingStatusButton.Importance = widget.HighImportance
		}
		buildingStatusButton.Refresh()
	}
	toggleBuildingStatus := func() {
		building, ok := findLauncherBuilding(buildings, buildingNameFromOption(buildingStatusSelect.Selected))
		if !ok {
			state.setStatus("건물 선택 필요", "상태를 변경할 건물을 선택하세요.")
			return
		}
		nextActive := !building.IsActive
		var err error
		if nextActive {
			err = state.runtime.Locations.ActivateBuilding(context.Background(), building.ID)
		} else {
			err = state.runtime.Locations.DeactivateBuilding(context.Background(), building.ID)
		}
		if err != nil {
			state.setStatus("건물 상태 변경 실패", "건물 상태 변경 실패: "+err.Error())
			return
		}
		status := "비활성화"
		if nextActive {
			status = "활성화"
		}
		refreshBuildings()
		buildingStatusSelect.SetSelected(buildingOptionForName(building.Name))
		state.refreshLocations()
		refreshBuildingStatusButton()
		state.setStatus("건물 상태 변경 완료", building.Name+" 건물을 "+status+"했습니다.")
	}
	buildingStatusButton = widget.NewButtonWithIcon("건물 선택 필요", theme.HelpIcon(), toggleBuildingStatus)
	buildingStatusSelect.OnChanged = func(string) {
		if building, ok := findLauncherBuilding(buildings, buildingNameFromOption(buildingStatusSelect.Selected)); ok {
			buildingEditEntry.SetText(building.Name)
		}
		refreshBuildingStatusButton()
	}
	refreshFloorStatusButton = func() {
		if floorStatusButton == nil {
			return
		}
		buildingID := buildingIDFromName(buildingNameFromOption(floorBuildingSelect.Selected))
		floor, ok := findBuildingFloor(buildingID, floorStatusSelect.Selected)
		if !ok {
			floorStatusButton.SetText("층 선택 필요")
			floorStatusButton.SetIcon(theme.HelpIcon())
			floorStatusButton.Importance = widget.LowImportance
			floorStatusButton.Refresh()
			return
		}
		if floor.IsActive {
			floorStatusButton.SetText("비활성화로 변경")
			floorStatusButton.SetIcon(theme.DeleteIcon())
			floorStatusButton.Importance = widget.LowImportance
		} else {
			floorStatusButton.SetText("활성화로 변경")
			floorStatusButton.SetIcon(theme.ConfirmIcon())
			floorStatusButton.Importance = widget.HighImportance
		}
		floorStatusButton.Refresh()
	}
	toggleFloorStatus := func() {
		buildingID := buildingIDFromName(buildingNameFromOption(floorBuildingSelect.Selected))
		floor, ok := findBuildingFloor(buildingID, floorStatusSelect.Selected)
		if !ok {
			state.setStatus("층 선택 필요", "상태를 변경할 층을 선택하세요.")
			return
		}
		nextActive := !floor.IsActive
		var err error
		if nextActive {
			err = state.runtime.Locations.ActivateBuildingFloor(context.Background(), floor.ID)
		} else {
			err = state.runtime.Locations.DeleteBuildingFloor(context.Background(), floor.ID)
		}
		if err != nil {
			state.setStatus("층 상태 변경 실패", "층 상태 변경 실패: "+err.Error())
			return
		}
		status := "비활성화"
		if nextActive {
			status = "활성화"
		}
		refreshBuildingFloors()
		floorStatusSelect.SetSelected(floorOptionForLabel(buildingID, floor.Label))
		state.setStatus("층 상태 변경 완료", floor.Building+" "+floor.Label+" 층을 "+status+"했습니다.")
	}
	floorStatusButton = widget.NewButtonWithIcon("층 선택 필요", theme.HelpIcon(), toggleFloorStatus)
	floorStatusSelect.OnChanged = func(string) {
		refreshFloorStatusButton()
	}
	buildingList = widget.NewList(
		func() int { return len(buildings) },
		func() fyne.CanvasObject {
			name := widget.NewLabel("")
			name.TextStyle = fyne.TextStyle{Bold: true}
			meta := widget.NewLabel("")
			return container.NewVBox(name, meta)
		},
		func(id widget.ListItemID, item fyne.CanvasObject) {
			box := item.(*fyne.Container)
			name := box.Objects[0].(*widget.Label)
			meta := box.Objects[1].(*widget.Label)
			building := buildings[id]
			name.SetText(building.Name)
			if building.IsActive {
				meta.SetText("사용")
			} else {
				meta.SetText("비활성")
			}
		},
	)
	buildingList.OnSelected = func(id widget.ListItemID) {
		if id < 0 || id >= len(buildings) {
			buildingStatusSelect.SetSelected("")
			refreshBuildingStatusButton()
			return
		}
		buildingStatusSelect.SetSelected(buildingOptionLabel(buildings[id]))
		buildingEditEntry.SetText(buildings[id].Name)
		refreshBuildingStatusButton()
		floorBuildingSelect.SetSelected(buildingOptionLabel(buildings[id]))
		refreshFloorControls()
	}
	filteredFloors := func() []domain.BuildingFloor {
		buildingID := buildingIDFromName(buildingNameFromOption(floorBuildingSelect.Selected))
		if buildingID <= 0 {
			return nil
		}
		items := make([]domain.BuildingFloor, 0, len(buildingFloors))
		for _, floor := range buildingFloors {
			if floor.BuildingID == buildingID {
				items = append(items, floor)
			}
		}
		return items
	}
	floorList = widget.NewList(
		func() int { return len(filteredFloors()) },
		func() fyne.CanvasObject {
			label := widget.NewLabel("")
			label.TextStyle = fyne.TextStyle{Bold: true}
			meta := widget.NewLabel("")
			return container.NewVBox(label, meta)
		},
		func(id widget.ListItemID, item fyne.CanvasObject) {
			floors := filteredFloors()
			if id < 0 || id >= len(floors) {
				return
			}
			box := item.(*fyne.Container)
			label := box.Objects[0].(*widget.Label)
			meta := box.Objects[1].(*widget.Label)
			floor := floors[id]
			label.SetText(floor.Label)
			if floor.IsActive {
				meta.SetText("사용")
			} else {
				meta.SetText("비활성")
			}
		},
	)
	floorList.OnSelected = func(id widget.ListItemID) {
		floors := filteredFloors()
		if id < 0 || id >= len(floors) {
			floorStatusSelect.SetSelected("")
			refreshFloorStatusButton()
			return
		}
		floorStatusSelect.SetSelected(floorOptionLabel(floors[id]))
		refreshFloorStatusButton()
	}
	refreshBuildingButton := widget.NewButtonWithIcon("새로고침", theme.ViewRefreshIcon(), refreshBuildings)
	refreshFloorButton := widget.NewButtonWithIcon("층 새로고침", theme.ViewRefreshIcon(), refreshBuildingFloors)
	buildingControlPane := container.NewVBox(
		widget.NewCard("건물 등록", "", container.NewBorder(nil, nil, nil, createBuildingButton, buildingNameEntry)),
		widget.NewCard("건물 수정", "", container.NewVBox(
			buildingStatusSelect,
			buildingEditEntry,
			container.NewHBox(updateBuildingButton, deleteBuildingButton),
		)),
		widget.NewCard("건물 상태 변경", "", container.NewVBox(
			buildingStatusButton,
		)),
		refreshBuildingButton,
		widget.NewCard("층 등록", "", container.NewVBox(
			floorBuildingSelect,
			container.NewBorder(nil, nil, nil, createFloorButton, floorNameEntry),
		)),
		widget.NewCard("층 상태 변경", "", container.NewVBox(
			floorStatusSelect,
			floorStatusButton,
		)),
		refreshFloorButton,
	)
	buildingListsPane := container.NewVSplit(
		widget.NewCard("건물 목록", "", buildingList),
		widget.NewCard("층 목록", "", floorList),
	)
	buildingListsPane.SetOffset(0.50)
	buildingPanel := container.NewHSplit(
		buildingControlPane,
		buildingListsPane,
	)
	buildingPanel.SetOffset(0.38)

	refreshRoles()
	refreshBuildings()
	refreshBuildingFloors()
	resetAddForm()
	resetEditForm()
	state.refreshLocations()

	tabs := container.NewAppTabs(
		container.NewTabItem("공간 추가", addPanel),
		container.NewTabItem("공간 수정", editPanel),
		container.NewTabItem("건물 관리", buildingPanel),
		container.NewTabItem("역할 관리", rolePanel),
	)
	tabs.SetTabLocation(container.TabLocationTop)
	return tabs
}

func buildLocationList(state *launcherState) *widget.List {
	return widget.NewList(
		func() int { return len(state.filteredLocations()) },
		func() fyne.CanvasObject {
			title := widget.NewLabel("")
			title.TextStyle = fyne.TextStyle{Bold: true}
			meta := widget.NewLabel("")
			meta.Wrapping = fyne.TextWrapWord
			return container.NewVBox(title, meta)
		},
		func(id widget.ListItemID, item fyne.CanvasObject) {
			box := item.(*fyne.Container)
			title := box.Objects[0].(*widget.Label)
			meta := box.Objects[1].(*widget.Label)
			location := state.filteredLocations()[id]
			title.SetText(location.Name)
			meta.SetText(fmt.Sprintf("%s / %s / %s", locationRoleLabel(locationRolesForUI(location)), locationStatusLabel(location), locationAddressLabel(location)))
		},
	)
}

func buildLocationInventoryList(state *launcherState) *widget.List {
	return widget.NewList(
		func() int { return len(state.locations) },
		func() fyne.CanvasObject {
			title := widget.NewLabel("")
			title.TextStyle = fyne.TextStyle{Bold: true}
			meta := widget.NewLabel("")
			meta.Wrapping = fyne.TextWrapWord
			return container.NewVBox(title, meta)
		},
		func(id widget.ListItemID, item fyne.CanvasObject) {
			if id < 0 || id >= len(state.locations) {
				return
			}
			box := item.(*fyne.Container)
			title := box.Objects[0].(*widget.Label)
			meta := box.Objects[1].(*widget.Label)
			location := state.locations[id]
			title.SetText(location.Name)
			meta.SetText(fmt.Sprintf("%s / %s / %s", locationRoleLabel(locationRolesForUI(location)), locationStatusLabel(location), locationAddressLabel(location)))
		},
	)
}

func locationRolesForUI(location domain.Location) []string {
	roles := make([]string, 0, len(location.Roles)+1)
	for _, role := range location.Roles {
		roles = append(roles, normalizeLauncherLocationRole(role))
	}
	if len(roles) == 0 {
		roles = append(roles, normalizeLauncherLocationRole(location.Type))
	}
	if location.IsClassroom && !hasLauncherLocationRole(roles, domain.LocationTypeClassroom) {
		roles = append(roles, domain.LocationTypeClassroom)
	}
	if len(roles) == 0 {
		roles = append(roles, "common")
	}
	return roles
}

func preferredLauncherLocationRole(roles []string) string {
	for _, role := range []string{domain.LocationTypeClassroom, domain.LocationTypeOffice, domain.LocationTypeReception, domain.LocationTypeHall, domain.LocationTypeStorage} {
		if hasLauncherLocationRole(roles, role) {
			return role
		}
	}
	return domain.LocationTypeOther
}

func normalizeLauncherLocationRole(role string) string {
	role = strings.TrimSpace(role)
	if role == "" || role == domain.LocationTypeOther {
		return "common"
	}
	return role
}

func hasLauncherLocationRole(roles []string, want string) bool {
	want = normalizeLauncherLocationRole(want)
	for _, role := range roles {
		if normalizeLauncherLocationRole(role) == want {
			return true
		}
	}
	return false
}

func uniqueLauncherRoles(roles []string) []string {
	unique := make([]string, 0, len(roles))
	for _, role := range roles {
		role = normalizeLauncherLocationRole(role)
		if role == "" || hasLauncherLocationRole(unique, role) {
			continue
		}
		unique = append(unique, role)
	}
	return unique
}

func withoutLauncherRole(roles []string, remove string) []string {
	remove = normalizeLauncherLocationRole(remove)
	filtered := make([]string, 0, len(roles))
	for _, role := range roles {
		if normalizeLauncherLocationRole(role) != remove {
			filtered = append(filtered, role)
		}
	}
	return filtered
}

func hasLauncherRoleName(roles []domain.LocationRole, name string) bool {
	name = normalizeLauncherLocationRole(name)
	for _, role := range roles {
		if normalizeLauncherLocationRole(role.Name) == name && role.IsActive {
			return true
		}
	}
	return false
}

func findLauncherLocationRole(roles []domain.LocationRole, name string) (domain.LocationRole, bool) {
	name = normalizeLauncherLocationRole(name)
	for _, role := range roles {
		if normalizeLauncherLocationRole(role.Name) == name {
			return role, true
		}
	}
	return domain.LocationRole{}, false
}

func locationRoleLabel(roles []string) string {
	labels := make([]string, 0, len(roles))
	for _, role := range roles {
		label := locationRoleOptionLabel(role)
		if !containsString(labels, label) {
			labels = append(labels, label)
		}
	}
	if len(labels) == 0 {
		return "공용"
	}
	return strings.Join(labels, ", ")
}

func locationRoleOptionLabel(role string) string {
	switch normalizeLauncherLocationRole(role) {
	case domain.LocationTypeClassroom:
		return "강의"
	case domain.LocationTypeOffice:
		return "사무"
	case domain.LocationTypeReception:
		return "접수"
	case domain.LocationTypeHall:
		return "행사"
	case domain.LocationTypeStorage:
		return "보관"
	case "common":
		return "공용"
	default:
		return strings.TrimSpace(role)
	}
}

func locationRoleManageOptionLabel(role domain.LocationRole) string {
	label := locationRoleOptionLabel(role.Name)
	if role.IsActive {
		return label
	}
	return label + " (비활성)"
}

func locationStatusLabel(location domain.Location) string {
	if location.IsActive {
		return "사용"
	}
	return "비활성"
}

func locationAddressLabel(location domain.Location) string {
	parts := []string{}
	if strings.TrimSpace(location.Building) != "" {
		parts = append(parts, location.Building)
	}
	if strings.TrimSpace(location.Floor) != "" {
		parts = append(parts, location.Floor)
	}
	if len(parts) == 0 {
		return "위치 미입력"
	}
	return strings.Join(parts, " ")
}

func buildingOptionLabel(building domain.Building) string {
	if building.IsActive {
		return building.Name
	}
	return building.Name + " (비활성)"
}

func findLauncherBuilding(buildings []domain.Building, name string) (domain.Building, bool) {
	name = strings.TrimSpace(name)
	for _, building := range buildings {
		if building.Name == name {
			return building, true
		}
	}
	return domain.Building{}, false
}

func locationDetailRow(title string, value *widget.Label) fyne.CanvasObject {
	name := widget.NewLabel(title)
	name.TextStyle = fyne.TextStyle{Bold: true}
	value.Wrapping = fyne.TextWrapWord
	return container.NewBorder(nil, nil, name, nil, value)
}

func containsString(values []string, want string) bool {
	for _, value := range values {
		if value == want {
			return true
		}
	}
	return false
}
